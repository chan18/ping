
mod ping;
mod capture;

fn main() {
    
    // capture::start();

    // requires admin access.
    ping::icmp();
}

