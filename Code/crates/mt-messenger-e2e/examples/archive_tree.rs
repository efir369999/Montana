// Демо Этапа 1: материализовать локальную иерархию «Монтана/Чаты/<чат>/».
// cargo run -p mt-messenger-e2e --example archive_tree -- <base>
use mt_messenger_e2e::archive::{
    history_key, seal_block, ArchiveStore, HistoryBlock, HistoryItem, DIR_IN, DIR_OUT,
};

fn main() {
    let base = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/Монтана".into());
    let st = ArchiveStore::open(&base).unwrap();
    let hk = history_key(&[0x55u8; 32]);
    let acct = [0x33u8; 32];

    for (chat, conv, msgs) in [
        (
            "Алиса",
            [0x22u8; 32],
            vec![(DIR_OUT, "привет"), (DIR_IN, "привет, как ты")],
        ),
        (
            "Боб",
            [0x44u8; 32],
            vec![(DIR_OUT, "фото ниже"), (DIR_IN, "ок")],
        ),
    ] {
        for (seq, (dir, text)) in msgs.iter().enumerate() {
            let block = HistoryBlock {
                block_seq: seq as u64,
                items: vec![HistoryItem {
                    conv_id: conv,
                    dir: *dir,
                    send_time: 1000 + seq as u64,
                    content: text.as_bytes().to_vec(),
                }],
            };
            st.append_block(chat, &seal_block(&hk, &acct, &block))
                .unwrap();
        }
        st.put_media(chat, "6c385ae2ef1c472b_demo.jpg", &hk, &acct, b"demo-media-bytes")
            .unwrap();
    }
    println!("Дерево создано в: {base}");
}
