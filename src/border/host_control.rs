//use std::marker::PhantomData;
//use futures::future::{ ok, FutureResult };
use std::collections::HashMap;
use log::info;
use url::{Host, HostAndPort, ParseError, Url};
use super::BorderControl;
use actix_web::{Error, HttpRequest};
use actix_web::dev::ServiceRequest;
use crate::border::{BorderControlBuilder, ClosedBorder};
use actix_http::error::{ErrorForbidden, ErrorNotImplemented, ErrorNotFound};
use actix_http::http::HeaderValue;

static DEFAULT: &str = "__default__";
type DestinationMap = HashMap<String, HostAndPort>;

#[derive(Clone)]
pub struct Destination( HostAndPort );

impl From<HostAndPort> for Destination {
    fn from( hp: HostAndPort ) -> Self {
        Destination( hp )
    }
}

impl From<Url> for Destination {
    fn from( url: Url ) -> Self {
        ( url.host().unwrap().to_owned(), url.port().unwrap_or( 80 ) ).into()
    }
}

impl From<(Host<String>, u16)> for Destination {
    fn from( host_port: (Host<String>, u16) ) -> Self {
        HostAndPort { host: host_port.0, port: host_port.1, }.into()
    }
}

pub struct HostControlBuilder {
    destinations: DestinationMap,
}

impl HostControlBuilder {
    pub fn new() -> Self {
        HostControlBuilder { destinations: HashMap::new(), }
    }

    pub fn with_default_destination<D: Into<Destination>>( mut self, dest: D ) -> Self {
        self.destinations.insert( DEFAULT.to_string(), dest.into().0 );
        self
    }

    pub fn with_named_destination<D: Into<Destination>>( mut self, name: &str, dest: D ) -> Self {
        self.destinations.insert( name.to_string(), dest.into().0 );
        self
    }

    fn is_closed( &self ) -> bool { self.destinations.is_empty() }

    fn has_only_default( &self ) -> bool {
        self.destinations.len() == 1 && self.destinations.contains_key( DEFAULT )
    }
}

impl BorderControlBuilder for HostControlBuilder {
    fn build( self ) -> Box<dyn BorderControl> {
        if self.is_closed() == true {
            Box::new( ClosedBorder::new() )
        } else if self.has_only_default() {
            Box::new(
                SingleHostBorder::new( self.destinations.get(DEFAULT).unwrap().clone() )
            )
        } else {
            Box::new( ManyHostsBorder::new( self.destinations ) )
        }
    }
}


#[derive(Clone)]
struct SingleHostBorder {
    destination: Destination,
}

impl SingleHostBorder {
    fn new<D: Into<Destination>>( destination: D ) -> Self {
        SingleHostBorder { destination: destination.into() }
    }

    fn from_host_and_port( host: Host<String>, port: u16 ) -> Self {
        SingleHostBorder::new( (host, port) )
    }
}

impl BorderControl for SingleHostBorder {
    fn request_visa(&self, req: &ServiceRequest ) -> Result<&HostAndPort, Error> {
        Ok( &self.destination.0 )
    }
}

#[derive(Clone)]
  struct ManyHostsBorder {
    destinations: DestinationMap,
    has_default: bool,
}

static HDR_X_DESTINATION: &str = "X-DESTINATION";

impl ManyHostsBorder {
    fn new( destinations: DestinationMap ) -> Self {
        let has_default = destinations.contains_key( DEFAULT );
        ManyHostsBorder { destinations, has_default, }
    }

    fn destination_for( &self, key: &str ) -> Result<&HostAndPort, Error> {
        self.destinations
            .get( key )
            .ok_or_else( || {
                ErrorNotFound( format!( "no egress destination identified for {}", key ) )
            } )
    }

    fn identify_destination( &self, req: &ServiceRequest ) -> Result<&HostAndPort, Error> {
        match req.headers().get( "X-DESTINATION" ) {
            Some(d) => self.destination_for( d.to_str().unwrap() ),

            None => {
                self.destination_for( DEFAULT )
                    .map_err( |_| {
                        ErrorNotFound( format!("no default egress destination for request {:?}", req.uri() ) )
                    } )
            }

//            Some(d) => Ok( d.to_str().unwrap() ),
//
//            None if self.has_default => Ok( DEFAULT ),
//
//            None => Err(
//                ErrorNotFound(format!("no egress destination found for request {:?}", req.uri()))
//            ),
        }
    }
}

impl BorderControl for ManyHostsBorder {
    fn request_visa(&self, req: &ServiceRequest ) -> Result<&HostAndPort, Error> {
        match req.headers().get( "X-DESTINATION" ) {
            Some(d) => self.destination_for(d.to_str().unwrap()),

            None => {
                self.destination_for(DEFAULT)
                    .map_err(|_| {
                        ErrorNotFound(format!("no default egress destination for request {:?}", req.uri()))
                    } )
            }
        }
    }
}







//#[derive(Clone)]
//pub struct HostControl {
//    destinations: HashMap<String, HostAndPort>,
//}
//
//impl HostControl {
//    #[allow(clippy::new_ret_no_self)]
//    pub fn new<C: Into<HostControlConfig>>( config: C ) -> HostControl {
//        HostControl { destinations: config.into().destinations, }
//    }
//}
//
//impl Default for HostControl {
//    fn default() -> Self {
//
//    }
//}
//
//impl BorderControl for HostControl {
//    fn requestVisa( &self, req: &ServiceRequest ) -> Result<HostAndPort, Error> {
//        let uri = req.uri();
//        info!( "uri={:?}, uri.host={:?}", uri, uri.host() );
//        let host = if let Some(h) = uri.host() {
//            info!( "host={:?}", h );
//            Host::parse( h )
//        } else {
//            Err( ParseError::EmptyHost )
//        }
//        .unwrap();
//
//        let port = uri.port_u16().unwrap();
//
//        info!( "host={:?} port={:?}", host, port );
//        let candidate = HostAndPort { host: host, port: port, };
//        info!( "candidate={:?}", candidate );
//
//        let result = match &self.whitelist {
//            None => true,
//            Some( wl ) => {
//                let mut contains = false;
//                for hp in wl.iter() {
//                    if (hp.host == candidate.host) && (hp.port == candidate.port) {
//                        contains = true;
//                        break;
//                    }
//                }
//                contains
//            },
//        };
//
//        Ok( result )
//    }
//}
