pub use crate::configuration::filter::FilterConfig;
use log::{debug, info, warn};
use proxy_wasm::{
    traits::{Context, HttpContext, RootContext},
    types::{ContextType, LogLevel},
};

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|context_id| -> Box<dyn RootContext> {
        Box::new(CacheFilterRoot {
            context_id,
            config: FilterConfig::default(),
        })
    });
}

struct CacheFilterRoot {
    context_id: u32,
    config: FilterConfig,
}

impl RootContext for CacheFilterRoot {
    fn on_vm_start(&mut self, _vm_configuration_size: usize) -> bool {
        info!("VM started");
        true
    }

    fn on_configure(&mut self, _config_size: usize) -> bool {
        //Check for the configuration passed by envoy.yaml
        let configuration: Vec<u8> = match self.get_configuration() {
            Some(c) => c,
            None => {
                warn!("Configuration missing. Please check the envoy.yaml file for filter configuration");
                return false;
            }
        };

        // Parse and store the configuration passed by envoy.yaml
        match serde_json::from_slice::<FilterConfig>(configuration.as_ref()) {
            Ok(config) => {
                debug!("configuring {}: {:?}", self.context_id, config);
                self.config = config;
                return true;
            }
            Err(e) => {
                warn!("Failed to parse envoy.yaml configuration: {:?}", e);
                return false;
            }
        }
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CacheFilter {
            config: self.config.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

impl Context for CacheFilterRoot {}

#[allow(dead_code)]
struct CacheFilter {
    config: FilterConfig,
}

impl HttpContext for CacheFilter {}

impl Context for CacheFilter {}

impl CacheFilter {}
