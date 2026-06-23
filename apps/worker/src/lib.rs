mod checkin;
mod command;
mod profile;
mod store;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use checkin::CheckInMessage;
use worker::*;

const IDENTITY_P12: &[u8] = include_bytes!("../assets/identity.p12");
const CA_DER: &[u8] = include_bytes!("../assets/ca.der");

fn var_or(ctx: &RouteContext<()>, key: &str, default: &str) -> String {
    ctx.var(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| default.to_string())
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::new()
        .get("/healthz", |_, _| Response::ok("ok"))
        .get_async("/readyz", |_, ctx| async move {
            match ctx.env.d1("DB") {
                Ok(_) => Response::ok("ready"),
                Err(_) => Response::error("no db binding", 503),
            }
        })
        .get_async("/enroll", handle_enroll)
        .post_async("/enroll", handle_enroll)
        .put_async("/checkin", handle_checkin)
        .post_async("/checkin", handle_checkin)
        .put_async("/server", handle_command)
        .post_async("/server", handle_command)
        .get_async("/admin/devices", handle_list_devices)
        .post_async("/admin/commands/:udid", handle_enqueue)
        .run(req, env)
        .await
}

async fn handle_enroll(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let base = var_or(&ctx, "BASE_URL", "https://mdm.stynx.app");
    let topic = var_or(&ctx, "MDM_TOPIC", "com.apple.mgmt.External.rustmdmpoc01");
    let password = var_or(&ctx, "IDENTITY_PASSWORD", "mdmidentity");

    let params = profile::EnrollmentParams {
        server_url: format!("{base}/server"),
        checkin_url: format!("{base}/checkin"),
        topic,
        pkcs12_der: IDENTITY_P12,
        pkcs12_password: password,
        ca_cert_der: CA_DER,
    };
    let body = profile::build_enrollment_profile(&params).map_err(Error::RustError)?;

    let h = Headers::new();
    h.set("content-type", "application/x-apple-aspen-config")?;
    h.set("content-disposition", "attachment; filename=\"enroll.mobileconfig\"")?;
    Ok(Response::from_bytes(body)?.with_headers(h))
}

async fn handle_checkin(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let bytes = req.bytes().await?;

    let msg = match CheckInMessage::from_plist(&bytes) {
        Ok(m) => m,
        Err(e) => return Response::error(format!("bad checkin plist: {e}"), 400),
    };

    match &msg {
        CheckInMessage::Authenticate { UDID, Topic } => {
            store::upsert_authenticate(&db, UDID, Topic.as_deref()).await?;
        }
        CheckInMessage::TokenUpdate {
            UDID,
            Token,
            PushMagic,
            Topic,
            UnlockToken,
        } => {
            let token_b64 = B64.encode(Token);
            let unlock = UnlockToken.as_ref().map(|u| B64.encode(u));
            store::update_token(&db, UDID, &token_b64, PushMagic, Topic, unlock.as_deref()).await?;
        }
        CheckInMessage::CheckOut { UDID } => {
            store::checkout(&db, UDID).await?;
        }
    }

    console_log!("checkin {} udid={}", msg.message_type(), msg.udid());
    Response::ok("")
}

async fn handle_command(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let bytes = req.bytes().await?;

    let resp = checkin::DeviceResponse::from_plist(&bytes).ok();

    if let Some(r) = &resp {
        if r.Status == "Acknowledged" {
            if let Some(uuid) = &r.CommandUUID {
                store::mark_acknowledged(&db, uuid).await.ok();
            }
        }
    }

    let udid = match resp.as_ref().and_then(|r| r.UDID.clone()) {
        Some(u) => u,
        None => return empty_200(),
    };

    match store::next_pending(&db, &udid).await? {
        Some(row) => {
            let env: command::CommandEnvelope =
                serde_json::from_str(&row.command_json).map_err(|e| Error::RustError(e.to_string()))?;
            let xml = env.to_xml().map_err(Error::RustError)?;
            store::mark_sent(&db, &row.command_uuid).await?;
            console_log!("delivering {} to {}", env.command.request_type(), udid);

            let h = Headers::new();
            h.set("content-type", "application/x-apple-aspen-mdm")?;
            Ok(Response::from_bytes(xml)?.with_headers(h))
        }
        None => empty_200(),
    }
}

async fn handle_list_devices(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let rows = store::list_devices(&db).await?;
    Response::from_json(&rows)
}

async fn handle_enqueue(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let udid = match ctx.param("udid") {
        Some(u) => u.clone(),
        None => return Response::error("missing udid", 400),
    };

    if !store::device_exists(&db, &udid).await? {
        return Response::error(format!("unknown device {udid}"), 404);
    }

    let admin: command::AdminCommand = match req.json().await {
        Ok(a) => a,
        Err(e) => return Response::error(format!("bad command json: {e}"), 400),
    };
    let cmd = admin.into_command().map_err(Error::RustError)?;
    let envelope = command::CommandEnvelope::new(cmd);
    let request_type = envelope.command.request_type().to_string();
    let command_uuid = envelope.command_uuid.to_string();
    let json = serde_json::to_string(&envelope).map_err(|e| Error::RustError(e.to_string()))?;

    store::enqueue(&db, &command_uuid, &udid, &request_type, &json).await?;
    console_log!("queued {request_type} for {udid} ({command_uuid})");

    Response::from_json(&serde_json::json!({
        "queued": command_uuid,
        "type": request_type,
        "udid": udid,
    }))
}

fn empty_200() -> Result<Response> {
    Response::ok("")
}
