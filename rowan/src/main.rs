
#[link_name="rowan_runtime"]
extern "C" {
    fn rowan_main();
}

pub fn main() {
    unsafe {
        rowan_main();
    }
}
