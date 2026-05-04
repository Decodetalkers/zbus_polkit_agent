use nix::unistd::getuid;
use zbus_polkit_agent::agent_session::PolkitAgengSession;
fn main() {
    let uid = getuid();
    // FOR EXAMPLE, uid is 1000
    let session = PolkitAgengSession::new(uid, None).unwrap();
    println!("{}", session.user_name());
}
