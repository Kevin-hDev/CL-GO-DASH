use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::net::lookup_host;

const BLOCKED_METADATA_IPS: &[IpAddr] = &[
    IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)),
    IpAddr::V4(Ipv4Addr::new(169, 254, 170, 2)),
    IpAddr::V4(Ipv4Addr::new(169, 254, 169, 253)),
    IpAddr::V4(Ipv4Addr::new(100, 100, 100, 200)),
    IpAddr::V6(Ipv6Addr::new(0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254)),
];

const BLOCKED_METADATA_HOSTS: &[&str] = &["metadata.google.internal", "metadata.goog"];

pub enum SsrfVerdict {
    Safe,
    Blocked(String),
}

pub async fn is_safe_url(url_str: &str, allow_private: bool) -> SsrfVerdict {
    let parsed = match url::Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return SsrfVerdict::Blocked("URL invalide".into()),
    };

    match parsed.scheme() {
        "http" | "https" => {}
        _ => return SsrfVerdict::Blocked("schéma non autorisé".into()),
    }

    let host = match parsed.host_str() {
        Some(h) => h.to_lowercase().trim_end_matches('.').to_string(),
        None => return SsrfVerdict::Blocked("hôte manquant".into()),
    };

    if BLOCKED_METADATA_HOSTS.iter().any(|h| host == *h) {
        return SsrfVerdict::Blocked("cloud metadata bloqué".into());
    }

    let port = parsed.port_or_known_default().unwrap_or(443);
    let lookup = format!("{}:{}", host, port);

    let addrs = match lookup_host(&lookup).await {
        Ok(a) => a.collect::<Vec<_>>(),
        Err(_) => return SsrfVerdict::Blocked("résolution DNS échouée".into()),
    };

    if addrs.is_empty() {
        return SsrfVerdict::Blocked("aucune adresse résolue".into());
    }

    for addr in &addrs {
        let ip = addr.ip();
        if is_metadata_ip(&ip) {
            return SsrfVerdict::Blocked("cloud metadata bloqué".into());
        }
        if !allow_private && is_blocked_ip(&ip) {
            return SsrfVerdict::Blocked("adresse privée bloquée".into());
        }
    }

    SsrfVerdict::Safe
}

pub fn is_metadata_ip(ip: &IpAddr) -> bool {
    BLOCKED_METADATA_IPS.contains(ip)
}

pub fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_multicast()
                || is_cgnat(v4)
                || is_reserved_v4(v4)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || is_ula(v6)
                || is_link_local_v6(v6)
                || is_mapped_private(v6)
        }
    }
}

fn is_cgnat(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (octets[1] & 0xC0) == 64
}

fn is_reserved_v4(ip: &Ipv4Addr) -> bool {
    let first = ip.octets()[0];
    first == 0 || first >= 240
}

fn is_ula(ip: &Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(ip: &Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

fn is_mapped_private(ip: &Ipv6Addr) -> bool {
    matches!(ip.to_ipv4_mapped(), Some(v4) if is_blocked_ip(&IpAddr::V4(v4)))
}
