use requiem_plugin::prelude::*;

struct Misbehaving;

impl Plugin for Misbehaving {
    fn before_send(_message: &OutgoingMessage) -> Verdict {
        panic!("boom");
    }

    fn transform(batch: &[IncomingMessage]) -> Vec<Verdict> {
        let mut n = batch.len() as u64;
        loop {
            n = n.wrapping_mul(6364136223846793005).wrapping_add(1);
            if n == 42 {
                return Vec::new();
            }
        }
    }

    fn settings_page() -> Node {
        let big = vec![1u8; 64 * 1024 * 1024];
        div().text(big.len().to_string())
    }
}

export_plugin!(Misbehaving);
