use ts_core::packet::*;

fn make_ts_packet(pid: u16, cc: u8) -> Vec<u8> {
    let mut pkt = vec![0u8; TS_PACKET_SIZE];
    pkt[0] = SYNC_BYTE;
    pkt[1] = ((pid >> 8) & 0x1F) as u8;
    pkt[2] = (pid & 0xFF) as u8;
    pkt[3] = 0x10 | (cc & 0x0F); // adaptation=01 (payload only), cc
    // fill payload with 0xFF
    for b in &mut pkt[4..] {
        *b = 0xFF;
    }
    pkt
}

#[test]
fn parse_basic_packet() {
    let data = make_ts_packet(0x0100, 5);
    let pkt = TsPacket::parse(&data).unwrap();

    assert_eq!(pkt.header.sync_byte, SYNC_BYTE);
    assert_eq!(pkt.header.pid, 0x0100);
    assert_eq!(pkt.header.continuity_counter, 5);
    assert!(!pkt.header.transport_error);
    assert!(!pkt.header.payload_unit_start);
    assert_eq!(pkt.header.adaptation_field_control, 1);
    assert!(pkt.adaptation.is_none());
    assert!(pkt.payload.is_some());
    assert_eq!(pkt.payload.unwrap().len(), 184);
}

#[test]
fn parse_sync_error() {
    let mut data = make_ts_packet(0x0100, 0);
    data[0] = 0x00;
    let err = TsPacket::parse(&data).unwrap_err();
    assert!(matches!(err, TsError::SyncError(0x00)));
}

#[test]
fn parse_too_short() {
    let data = vec![SYNC_BYTE; 10];
    let err = TsPacket::parse(&data).unwrap_err();
    assert!(matches!(err, TsError::TooShort(10)));
}

#[test]
fn parse_null_packet() {
    let data = make_ts_packet(0x1FFF, 0);
    let pkt = TsPacket::parse(&data).unwrap();
    assert_eq!(pkt.header.pid, 0x1FFF);
}

#[test]
fn parse_adaptation_field_with_pcr() {
    let mut data = vec![0u8; TS_PACKET_SIZE];
    data[0] = SYNC_BYTE;
    data[1] = 0x01; // PID = 0x0100
    data[2] = 0x00;
    data[3] = 0x30; // adaptation + payload, cc=0

    // adaptation field
    data[4] = 7; // af length
    data[5] = 0x10; // PCR flag set
    // PCR bytes (base=12345, ext=100)
    // base = 12345, ext = 100
    // raw_pcr = base * 300 + ext = 3703600
    let base: u64 = 12345;
    let ext: u16 = 100;
    data[6] = ((base >> 25) & 0xFF) as u8;
    data[7] = ((base >> 17) & 0xFF) as u8;
    data[8] = ((base >> 9) & 0xFF) as u8;
    data[9] = ((base >> 1) & 0xFF) as u8;
    data[10] = (((base & 1) << 7) as u8) | 0x7E | ((ext >> 8) as u8 & 0x01);
    data[11] = (ext & 0xFF) as u8;

    let pkt = TsPacket::parse(&data).unwrap();
    let af = pkt.adaptation.unwrap();
    assert_eq!(af.pcr, Some(base * 300 + ext as u64));
    assert!(!af.discontinuity);
    assert!(!af.random_access);
}
