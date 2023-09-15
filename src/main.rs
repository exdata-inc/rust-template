#[macro_use]
extern crate log;
extern crate env_logger as logger;

// Standard Library
use std::{env, result};
use std::sync::Arc;
use std::convert::TryInto;

// External Library
use chrono::Local;
use clap::{Parser, builder::Resettable};
use serde_json::json;
use tokio::sync::{Mutex, RwLock};
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
static SX_SERVICE_CLIENT: async_once_cell::OnceCell<Arc<RwLock<sxutil::SXServiceClient>>> = async_once_cell::OnceCell::new();

static LOOP_FLAG: once_cell::sync::Lazy<Mutex<Option<Arc<Mutex<bool>>>>> = once_cell::sync::Lazy::new(|| Mutex::from(None));

async fn subscribe_mbus(mbus_id: u64, mbcb: fn(&sxutil::SXServiceClient, api::MbusMsg)) {
    SX_SERVICE_CLIENT.get().unwrap().read().await.subscribe_mbus(mbus_id, mbcb).await;
}

fn mbus_callback_notifyer(_sxsv_clt: &sxutil::SXServiceClient, mmsg: api::MbusMsg) {
    info!("{:?}", mmsg);
}

fn mbus_callback_subscriber(_sxsv_clt: &sxutil::SXServiceClient, mmsg: api::MbusMsg) {
    info!("{:?}", mmsg);
}

async fn supply_callback_notifyer(_sxsv_clt: &sxutil::SXServiceClient, sp: api::Supply) {
    match sp.supply_name.as_str() {
        "Template:ProposeSupply" => {
            let v: serde_json::Value = serde_json::from_str(sp.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Supply Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "Template:ProposeSupply" => {
                    info!("Rust Template SubscribeDemand Node's ProposeSupply Message: {:?}, {:?}", v, sp);
                    tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                    let sx_res = _sxsv_clt.select_supply(sp).await;
                    if sx_res.is_some() && sx_res.unwrap() > 0 {
                        info!("Sent SelectSupply msg and confirmed! Start Subscribing Mbus... mbus_id:{} self.id:{}", sx_res.unwrap(), _sxsv_clt.client_id);
                        tokio::spawn(subscribe_mbus(sx_res.unwrap(), mbus_callback_notifyer));
                    } else {
                        error!("Failed to send SelectSupply msg");
                    }
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

async fn demand_callback_notifyer(_sxsv_clt: &sxutil::SXServiceClient, dm: api::Demand) {
    match dm.demand_name.as_str() {
        "Template:ProposeDemand" => {
            let v: serde_json::Value = serde_json::from_str(dm.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Demand Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "Template:ProposeDemand" => {
                    info!("Rust Template SubscribeSupply Node's ProposeDemand Message: {:?}, {:?}", v, dm);
                    tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                    let sx_res = _sxsv_clt.select_demand(dm).await;
                    if sx_res.is_some() && sx_res.unwrap() > 0 {
                        info!("Sent SelectDemand msg and confirmed! Start Subscribing Mbus... mbus_id:{} self.id:{}", sx_res.unwrap(), _sxsv_clt.client_id);
                        tokio::spawn(subscribe_mbus(sx_res.unwrap(), mbus_callback_notifyer));
                    } else {
                        error!("Failed to send SelectDemand msg");
                    }
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
        "Template:NotifySupply" => {
            let v: serde_json::Value = serde_json::from_str(sp.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Supply Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "Template:NotifySupply" => {
                    info!("Rust Template NotifySupply Node's NotifySupply Message: {:?}, {:?}", v, sp);
                    tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                    let msg = json!({
                        "@context": {
                            "schema": "https://schema.org/"
                        },
                        "@id": "supply_node",
                        "@type": "Template:ProposeDemand",
                        "schema:name": format!("Demand Message from SubscribeSupply mode node for Supply[{}]", v["schema:identifier"]),
                        "schema:identifier": v["schema:identifier"],
                    }).to_string();
                    let sx_res = _sxsv_clt.propose_demand(sxutil::DemandOpts{
                        id: 0,
                        target: 0,
                        name: "Template:ProposeDemand".to_string(),
                        json: msg.clone(),
                        cdata: api::Content { entity: vec![] },
                    }).await;
                    if sx_res > 0 {
                        info!("Sent ProposeDemand msg, len: {}, id: {}", msg.len(), sx_res);
                    } else {
                        error!("Failed to send ProposeDemand msg");
                    }
                }
                &_ => {
                    warn!("Ignore Supply: {:?} {:?}", sp.supply_name.as_str(), v);
                }
            }
        }
        &_ => {
            info!("Possibly Rust Template NotifySupply Node's SelectDemand Message: {:?} {:?}", sp.supply_name.as_str(), sp);
            if _sxsv_clt.ni.as_ref().unwrap().read().await.node_state.proposed_demand_index(sp.target_id) != -1 {
                let mut confirm_result = false;
                match _sxsv_clt.confirm(sp.id, sp.target_id).await {
                    Ok(_) => {
                        info!("Confirmed!");
                        confirm_result = true;
                    },
                    Err(err) => {
                        info!("Error: {:?}", err);
                    },
                };
                if confirm_result {
                    //tokio::spawn(subscribe_mbus(sp.mbus_id, mbus_callback_subscriber));
                    tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                    info!("Sending Mbus msg... mbus_id:{}, self.id:{}, sp:{:?}", sp.mbus_id, _sxsv_clt.client_id, sp);
                    _sxsv_clt.send_mbus_msg(sp.mbus_id, api::MbusMsg{
                        msg_id: 0,
                        sender_id: sp.target_id,
                        target_id: sp.sender_id,
                        mbus_id: sp.mbus_id,
                        msg_type: 0,
                        msg_info: "data".to_string(),
                        arg_json: "{}".to_string(),
                        cdata: Some(api::Content { entity: vec![0,1,2] }),
                    }).await;
                    info!("Sent Mbus msg!");
                }
            } else {
                info!("unmatch id. sp.target_id:{}", sp.target_id);
            }
        }
    }
}

async fn demand_callback_echo(_sxsv_clt: &sxutil::SXServiceClient, dm: api::Demand) {
    match dm.demand_name.as_str() {
        "Template:NotifyDemand" => {
            let v: serde_json::Value = serde_json::from_str(dm.arg_json.as_str()).unwrap();
            if v["@type"].as_str().is_none() {
                error!("Unknown Demand Type! {:?}", v);
            }
            match v["@type"].as_str().unwrap() {
                "Template:NotifyDemand" => {
                    info!("Rust Template NotifyDemand Node's NotifyDemand Message: {:?}", v);
                    tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                    let msg = json!({
                        "@context": {
                            "schema": "https://schema.org/"
                        },
                        "@id": "supply_node",
                        "@type": "Template:ProposeSupply",
                        "schema:name": format!("Supply Message from SubscribeDemand mode node for Demand[{}]", v["schema:identifier"]),
                        "schema:identifier": v["schema:identifier"],
                    }).to_string();
                    let sx_res = _sxsv_clt.propose_supply(&sxutil::SupplyOpts{
                        id: 0,
                        target: 0,
                        name: "Template:ProposeSupply".to_string(),
                        json: msg.clone(),
                        cdata: api::Content { entity: vec![] },
                    }).await;
                    if sx_res > 0 {
                        info!("Sent ProposeSupply msg, len: {}, id: {}", msg.len(), sx_res);
                    } else {
                        error!("Failed to send ProposeSupply msg");
                    }
                }
                &_ => {
                    warn!("Ignore Demand: {:?} {:?}", dm.demand_name.as_str(), v);
                }
            }
        }
        &_ => {
            info!("Possibly Rust Template NotifyDemand Node's SelectSupply Message: {:?} {:?}", dm.demand_name.as_str(), dm);
            if _sxsv_clt.ni.as_ref().unwrap().read().await.node_state.proposed_supply_index(dm.target_id) != -1 {
                let mut confirm_result = false;
                match _sxsv_clt.confirm(dm.id, dm.target_id).await {
                    Ok(_) => {
                        info!("Confirmed!");
                        confirm_result = true;
                    },
                    Err(err) => {
                        info!("Error: {:?}", err);
                    },
                };
                if confirm_result {
                    //tokio::spawn(subscribe_mbus(dm.mbus_id, mbus_callback_subscriber));
                    tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
                    info!("Sending Mbus msg... mbus_id:{}, self.id:{}, target:{}", dm.mbus_id, _sxsv_clt.client_id, dm.sender_id);
                    _sxsv_clt.send_mbus_msg(dm.mbus_id, api::MbusMsg{
                        msg_id: 0,
                        sender_id: 0,
                        target_id: dm.sender_id,
                        mbus_id: dm.mbus_id,
                        msg_type: 0,
                        msg_info: "data".to_string(),
                        arg_json: "{}".to_string(),
                        cdata: Some(api::Content { entity: vec![0,1,2] }),
                    }).await;
                    info!("Sent Mbus msg!");
                }
            } else {
                info!("unmatch id. dm.target_id:{}", dm.target_id);
            }
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
    SX_SERVICE_CLIENT.get_or_init(async {
        let sx_service_client = sxutil::new_sx_service_client(client.unwrap(), synerex_proto::JSON_DATA_SVC, arg_json).await;
        Arc::new(RwLock::from(sx_service_client)) 
    }).await;
    debug!("SXServiceClient: {:?}", SX_SERVICE_CLIENT.get().unwrap().read().await);

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
                    "@type": format!("Template:Notify{}", &*args.msg_type),
                    "schema:name": format!("{} Message from Notify{} Mode Node. Count = {}", &*args.msg_type, &*args.msg_type, i),
                    "schema:identifier": i,
                }).to_string();
                
                match &*args.msg_type {
                    "Supply" => {
                        info!("Sending NotifySupply msg[{}]", i);
                        let sx_res = SX_SERVICE_CLIENT.get().unwrap().read().await.notify_supply(sxutil::SupplyOpts{
                            id: 0,
                            target: 0,
                            name: "Template:NotifySupply".to_string(),
                            json: msg.clone(),
                            cdata: api::Content { entity: vec![] },
                        }).await;
                        if sx_res.is_some() {
                            info!("Sent NotifySupply msg[{}], {}", i, sx_res.unwrap());
                            if LOOP_FLAG.lock().await.as_ref().is_none() {
                                let dmcb: sxutil::DemandHandler = Box::pin(|sxsv_clt, dm| {
                                    Box::pin(demand_callback_notifyer(sxsv_clt, dm))
                                });
                                *LOOP_FLAG.lock().await = Some(sxutil::simple_subscribe_demand(
                                    Arc::clone(&*SX_SERVICE_CLIENT.get().unwrap()), 
                                    dmcb
                                ));
                            }     
                        } else {
                            error!("Failed to send NotifySupply msg");
                        }
                    }
                    "Demand" => {
                        info!("Sending NotifyDemand msg[{}]", i);
                        let sx_res = SX_SERVICE_CLIENT.get().unwrap().read().await.notify_demand(sxutil::DemandOpts{
                            id: 0,
                            target: 0,
                            name: "Template:NotifyDemand".to_string(),
                            json: msg.clone(),
                            cdata: api::Content { entity: vec![] },
                        }).await;
                        if sx_res.is_some() {
                            info!("Sent NotifyDemand msg[{}], {}", i, sx_res.unwrap());
                            if LOOP_FLAG.lock().await.as_ref().is_none() {
                                let spcb: sxutil::SupplyHandler = Box::pin(|sxsv_clt, sp| {
                                    Box::pin(supply_callback_notifyer(sxsv_clt, sp))
                                });
                                *LOOP_FLAG.lock().await = Some(sxutil::simple_subscribe_supply(
                                    Arc::clone(&*SX_SERVICE_CLIENT.get().unwrap()), 
                                    spcb
                                ));
                            }     
                        } else {
                            error!("Failed to send NotifyDemand msg");
                        }
                    }
                    &_ => {
                        error!("Unknown Message Type!");
                    }    
                }
                i += 1;
            }
        }
        "subscribe" => {
            // Subscribe Mode

            match &*args.msg_type {
                "Supply" => {
                    let spcb: sxutil::SupplyHandler = Box::pin(|sxsv_clt, sp| {
                        Box::pin(supply_callback_echo(sxsv_clt, sp))
                    });

                    let _loop_flag = sxutil::simple_subscribe_supply(
                        Arc::clone(&*SX_SERVICE_CLIENT.get().unwrap()), 
                        spcb
                    );
                }
                "Demand" => {
                    let dmcb: sxutil::DemandHandler = Box::pin(|sxsv_clt, dm| {
                        Box::pin(demand_callback_echo(sxsv_clt, dm))
                    });

                    let _loop_flag = sxutil::simple_subscribe_demand(
                        Arc::clone(&*SX_SERVICE_CLIENT.get().unwrap()), 
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
        &_ => {
            error!("Unknown Mode!");
        }
    }

    let finish_time = Local::now();
    info!("Finished Program at {}", finish_time.format("%F %T %:z"));
    Ok(())
}
