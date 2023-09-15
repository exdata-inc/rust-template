#[macro_use]
extern crate log;
extern crate env_logger as logger;

// Standard Library
use std::env;
use std::sync::Arc;
use std::convert::TryInto;

// External Library
use chrono::Local;
use clap::Parser;
use serde_json::json;
use tokio::sync::Mutex;
use prost::Message;

// Synerex Library
use synerex_proto;
use synerex_api::api;
use dbp_schema::dbp_schema::RealWorldDataset;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'n', long = "node_addr", value_name = "Node Server Address", default_value = "localhost:9990")]
    node_addr: String,
    #[arg(short = 's', long = "sx_addr", value_name = "Synerex Server Address", default_value = "use_default")]
    sx_addr: String,
    #[arg(short = 'm', long = "mode", value_name = "Synerex Provider Mode", default_value = "notify")]
    mode: String,
    #[arg(short = 't', long = "msg_type", value_name = "Synerex Message Mode", default_value = "supply")]
    msg_type: String,
    #[arg(short = 'l', long = "log_level", value_name = "Log Level (ERROR, INFO, DEBUG)", default_value = "INFO")]
    log_level: String,
}

static SX_ADDR: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();
static SX_SERVICE_CLIENT_1: async_once_cell::OnceCell<Arc<Mutex<sxutil::SXServiceClient>>> = async_once_cell::OnceCell::new();
static SX_SERVICE_CLIENT_2: async_once_cell::OnceCell<Arc<Mutex<sxutil::SXServiceClient>>> = async_once_cell::OnceCell::new();

async fn supply_callback(_sxsv_clt: &sxutil::SXServiceClient, sp: api::Supply) {
    match sp.supply_name.as_str() {
        "Rust:Template" => {
            let v: serde_json::Value = serde_json::from_str(sp.arg_json.as_str()).unwrap();
            if sp.cdata.is_some() {
                let cdata = sp.cdata.unwrap().entity;
                // vec_try_into_prost!(Hello);
                // let rwdataset_result: Result<RealWorldDataset, prost::DecodeError> = cdata.try_into();
            }
            if v["@type"].as_str().is_none() {
                error!("Unknown Supply Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "rust:template" => {
                    info!("Rust Template Supply: {:?}", v);
                }
                &_ => {
                    warn!("Unknown Supply: {:?} {:?}", sp.supply_name.as_str(), v);
                }
            }
        }
        "Rust:TemplateEcho" => {
            let v: serde_json::Value = serde_json::from_str(sp.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Supply Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "rust:template-echo" => {
                    info!("Rust Template Echo Supply: {:?}", v);
                }
                &_ => {
                    warn!("Unknown Supply: {:?} {:?}", sp.supply_name.as_str(), v);
                }
            }
        }
        &_ => {
            warn!("Unknown Supply: {:?}", sp.supply_name.as_str());
        }
    }
}

async fn demand_callback(_sxsv_clt: &sxutil::SXServiceClient, dm: api::Demand) {
    match dm.demand_name.as_str() {
        "Rust:Template" => {
            let v: serde_json::Value = serde_json::from_str(dm.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Demand Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "rust:template" => {
                    info!("Rust Template Demand: {:?}", v);
                }
                &_ => {
                    warn!("Unknown Demand: {:?} {:?}", dm.demand_name.as_str(), v);
                }
            }
        }
        "Rust:TemplateEcho" => {
            let v: serde_json::Value = serde_json::from_str(dm.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Demand Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "rust:template-echo" => {
                    info!("Rust Template Echo Demand: {:?}", v);
                }
                &_ => {
                    warn!("Unknown Demand: {:?} {:?}", dm.demand_name.as_str(), v);
                }
            }
        }
        &_ => {
            warn!("Unknown Demand: {:?}", dm.demand_name.as_str());
        }
    }
}

async fn supply_callback_echo(_sxsv_clt: &sxutil::SXServiceClient, sp: api::Supply) {
    match sp.supply_name.as_str() {
        "Rust:Template" => {
            let v: serde_json::Value = serde_json::from_str(sp.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Supply Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "rust:template" => {
                    info!("Rust Template Supply: {:?}", v);
                    let msg = json!({
                        "@context": {
                            "schema": "https://schema.org/"
                        },
                        "@id": "supply_node",
                        "@type": "rust:template-echo",
                        "schema:name": format!("Message from Supply-echo Mode Node.")
                    }).to_string();
                    let sx_res = _sxsv_clt.notify_supply(sxutil::SupplyOpts{
                        id: 0,
                        target: 0,
                        name: "Rust:TemplateEcho".to_string(),
                        json: msg.clone(),
                        cdata: api::Content { entity: vec![] },
                    }).await;
                    if sx_res.is_some() {
                        info!("Sent NotifySupply msg, len: {}", msg.len());
                    } else {
                        error!("Failed to send NotifySupply msg");
                    }
                }
                &_ => {
                    warn!("Ignore Supply: {:?} {:?}", sp.supply_name.as_str(), v);
                }
            }
        }
        &_ => {
            warn!("Ignore Supply: {:?}", sp.supply_name.as_str());
        }
    }
}

async fn demand_callback_echo(_sxsv_clt: &sxutil::SXServiceClient, dm: api::Demand) {
    match dm.demand_name.as_str() {
        "Rust:Template" => {
            let v: serde_json::Value = serde_json::from_str(dm.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Demand Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "rust:template" => {
                    info!("Rust Template Demand: {:?}", v);
                    let msg = json!({
                        "@context": {
                            "schema": "https://schema.org/"
                        },
                        "@id": "supply_node",
                        "@type": "rust:template-echo",
                        "schema:name": format!("Message from Demand-echo Mode Node.")
                    }).to_string();
                    let sx_res = _sxsv_clt.notify_demand(sxutil::DemandOpts{
                        id: 0,
                        target: 0,
                        name: "Rust:TemplateEcho".to_string(),
                        json: msg.clone(),
                        cdata: api::Content { entity: vec![] },
                    }).await;
                    if sx_res.is_some() {
                        info!("Sent NotifyDemand msg, len: {}", msg.len());
                    } else {
                        error!("Failed to send NotifyDemand msg");
                    }
                }
                &_ => {
                    warn!("Ignore Demand: {:?} {:?}", dm.demand_name.as_str(), v);
                }
            }
        }
        &_ => {
            warn!("Ignore Demand: {:?}", dm.demand_name.as_str());
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env::set_var("RUST_LOG", args.log_level);
    logger::init();

    let start_time = Local::now();
    info!("Started Program at {}", start_time.format("%F %T %:z"));

    debug!("Using Synerex Config: Node: {}, Synerex: {}", args.node_addr, args.sx_addr);

    // Set your target channel types and nm here.
    let channel_types = vec![synerex_proto::JSON_DATA_SVC];
    let nm = String::from(format!("RustTemp:{}", args.mode));

    // Register node and acquire Synerex server address.
    let sx_addr = match sxutil::register_node(String::from("http://") + args.node_addr.as_str(), nm.clone(), channel_types, None).await {
        Ok(srv) => {
            tokio::spawn(sxutil::start_keep_alive_with_cmd(None));
            if args.sx_addr != "use_default" { args.sx_addr }
            else { srv }
        },
        Err(err) => {
            error!("Failed to connect Synerex-Node server. {}", err);
            err
        }
    };
	let set_result = SX_ADDR.set(sx_addr);
    debug!("SX_ADDR: {:?} ({:?})", SX_ADDR.get(), set_result);

    // Connect to Synerex server.
	let client = sxutil::grpc_connect_server(String::from("http://") + SX_ADDR.get().unwrap().as_str()).await;  // sxServerAddress
    debug!("SXSynerexClient: {:?} {}", client, SX_ADDR.get().unwrap().as_str());
    if client.is_none() {
        error!("Failed to connect Synerex server.");
        std::process::exit(1);
    }

    // Initialize Synerex service client.
    let arg_json = String::from(format!("{{{}}}", nm));
    SX_SERVICE_CLIENT_1.get_or_init(async {
        let sx_service_client = sxutil::new_sx_service_client(client.unwrap(), synerex_proto::JSON_DATA_SVC, arg_json).await;
        Arc::new(Mutex::new(sx_service_client)) 
    }).await;
    debug!("SXServiceClient: {:?}", SX_SERVICE_CLIENT_1.get().unwrap().lock().await);

    match &*args.mode {
        "notify" => {
            // Notify Mode
            let mut i = 0;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let msg = json!({
                    "@context": {
                        "schema": "https://schema.org/"
                    },
                    "@id": "notify_node",
                    "@type": "rust:template",
                    "schema:name": format!("Message from Notify Mode Node. Count = {}", i)
                }).to_string();
                if i % 2 == 0 {
                    let sx_res = SX_SERVICE_CLIENT_1.get().unwrap().lock().await.notify_supply(sxutil::SupplyOpts{
                        id: 0,
                        target: 0,
                        name: "Rust:Template".to_string(),
                        json: msg.clone(),
                        cdata: api::Content { entity: vec![] },
                    }).await;
                    if sx_res.is_some() {
                        info!("Sent NotifySupply msg[{}], {}", i, sx_res.unwrap());
                    } else {
                        error!("Failed to send NotifySupply msg");
                    }
                } else {
                    let sx_res = SX_SERVICE_CLIENT_1.get().unwrap().lock().await.notify_demand(sxutil::DemandOpts{
                        id: 0,
                        target: 0,
                        name: "Rust:Template".to_string(),
                        json: msg.clone(),
                        cdata: api::Content { entity: vec![] },
                    }).await;
                    if sx_res.is_some() {
                        info!("Sent NotifyDemand msg[{}], {}", i, sx_res.unwrap());
                    } else {
                        error!("Failed to send NotifyDemand msg");
                    }
                }
                i += 1;
            }
        }
        "subscribe" => {
            // Subscribe Mode

            match &*args.msg_type {
                "supply" => {
                    let spcb: sxutil::SupplyHandler = Box::pin(|sxsv_clt, sp| {
                        Box::pin(supply_callback(sxsv_clt, sp))
                    });

                    let _loop_flag = sxutil::simple_subscribe_supply(
                        Arc::clone(&*SX_SERVICE_CLIENT_1.get().unwrap()), 
                        spcb
                    );
                }
                "demand" => {
                    let dmcb: sxutil::DemandHandler = Box::pin(|sxsv_clt, dm| {
                        Box::pin(demand_callback(sxsv_clt, dm))
                    });

                    let _loop_flag = sxutil::simple_subscribe_demand(
                        Arc::clone(&*SX_SERVICE_CLIENT_1.get().unwrap()), 
                        dmcb
                    );
                }
                &_ => {
                    error!("Unknown Message Type!");
                }
            }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
        "echo" => {
            // Echo Mode

            // Connect to Synerex server.
            let client = sxutil::grpc_connect_server(String::from("http://") + SX_ADDR.get().unwrap().as_str()).await;  // sxServerAddress
            debug!("SXSynerexClient2: {:?} {}", client, SX_ADDR.get().unwrap().as_str());
            if client.is_none() {
                error!("Failed to connect Synerex server.");
                std::process::exit(1);
            }

            // Initialize Synerex service client.
            let arg_json = String::from(format!("{{{}}}", nm));
            SX_SERVICE_CLIENT_2.get_or_init(async {
                let sx_service_client = sxutil::new_sx_service_client(client.unwrap(), synerex_proto::JSON_DATA_SVC, arg_json).await;
                Arc::new(Mutex::new(sx_service_client)) 
            }).await;
            debug!("SXServiceClient2: {:?}", SX_SERVICE_CLIENT_2.get().unwrap().lock().await);

            let spcb: sxutil::SupplyHandler = Box::pin(|sxsv_clt, sp| {
                Box::pin(supply_callback_echo(sxsv_clt, sp))
            });

            let dmcb: sxutil::DemandHandler = Box::pin(|sxsv_clt, dm| {
                Box::pin(demand_callback_echo(sxsv_clt, dm))
            });

            let _loop_flag_sp = sxutil::simple_subscribe_supply(
                Arc::clone(&*SX_SERVICE_CLIENT_1.get().unwrap()), 
                spcb
            );

            let _loop_flag_dm = sxutil::simple_subscribe_demand(
                Arc::clone(&*SX_SERVICE_CLIENT_2.get().unwrap()), 
                dmcb
            );

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
        &_ => {
            error!("Unknown Mode!");
        }
    }

    let finish_time = Local::now();
    info!("Finished Program at {}", finish_time.format("%F %T %:z"));
    Ok(())
}
