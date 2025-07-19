extern "C" {
    #[link_name = "rowan_main"]
    pub fn rowan_main();
}

fn main() {
    unsafe {
        rowan_main();
    }
}
