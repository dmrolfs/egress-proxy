use log::{info, error};
use clap::{value_t, Arg, ArgMatches};
use url::Url;
use std::io::{Result, ErrorKind::NotFound};
use std::net::{IpAddr, SocketAddr, TcpListener, ToSocketAddrs};
use std::str::FromStr;
use listenfd::ListenFd;

const PROTOCOL: &str = "http";

const LISTEN_HOST: &str = "listen_host";
const LISTEN_PORT: &str = "listen_port";
const FORWARD_HOST: &str = "forward_host";
const FORWARD_PORT: &str = "forward_port";


#[derive(Clone,Debug)]
pub struct Config {
    pub listen_socket_address: Option<SocketAddr>,
    pub forward_url: Url,
}

impl Config {
    pub fn from_args() -> Config {
        let matches = arg_matches();
        let lhost = matches.value_of( LISTEN_HOST );
        let lport = Self::match_listen_port( &matches );

        let socket = match (lhost, lport) {
            (Some(h), Some(p)) => {
                let ip = IpAddr::from_str(h ).unwrap();
                Some( SocketAddr::new(ip, p ) )
            },
            _ => None
        };

        let fhost = matches.value_of( FORWARD_HOST ).unwrap();
        let fport = value_t!( matches, FORWARD_PORT, u16 ).unwrap_or_else(|e| e.exit() );
        let furl = Url::parse(
            &format!(
                "{}://{}",
                PROTOCOL,
                (fhost, fport)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap()
            )
        ).unwrap();

        Config {
            listen_socket_address: socket,
            forward_url: furl,
        }
    }

    fn match_listen_port( m: &ArgMatches ) -> Option<u16> {
        if m.is_present( LISTEN_PORT ) {
            let p = value_t!( m, LISTEN_PORT, u16 ).unwrap_or_else( |e| {
                error!( "failed to parse LISTEN PORT value: {}", e );
                e.exit();
            } );

            Some( p )
        } else {
            None
        }
    }

    pub fn tcp_listener( &self ) -> Result<TcpListener> {
        self.listen_socket_address
            .map( |sa| TcpListener::bind( sa ).map( |l| Some(l)) )
            .unwrap_or_else(
                || {
                    info!( "listen socket address not specified, seeking system listener...");
                    let mut listenfd = ListenFd::from_env();
                    listenfd.take_tcp_listener( 0 )
                }
            )
            .and_then(
                |l| match l {
                    Some(l) => {
                        info!( "listening on {}", l.local_addr().unwrap() );
                        Ok(l)
                    },
                    None => Err( NotFound.into() )
                }
            )
    }
}

fn arg_matches<'a>() -> ArgMatches<'a> {
    clap::App::new( "HTTP Egress Proxy" )
        .arg(
            Arg::with_name( LISTEN_HOST )
                .takes_value( true )
                .value_name( "LISTEN HOST" )
                .short( "H" )
                .long( "lhost" )
                .required( false ),
        )
        .arg(
            Arg::with_name( LISTEN_PORT )
                .takes_value( true )
                .value_name( "LISTEN PORT" )
                .short( "P" )
                .long( "lport" )
                .required( false ),
        )
        .arg(
            Arg::with_name( FORWARD_HOST )
                .takes_value( true )
                .value_name( "FORWARD HOST" )
                .short( "h" )
                .long( "fhost" )
                .required( true ),
        )
        .arg(
            Arg::with_name( FORWARD_PORT )
                .takes_value( true )
                .value_name( "FORWARD PORT" )
                .short( "p" )
                .long( "fport" )
                .required( true ),
        )
        .get_matches()
}