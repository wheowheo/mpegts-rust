use ts_core::packet::{TsPacket, TS_PACKET_SIZE};
use ts_core::packet_builder::{TsPacketBuilder, build_null_packet};
use ts_core::psi_builder::{PatBuilder, PmtBuilder};
use ts_core::psi::{PsiSection, pat::Pat, pmt::Pmt};
use ts_core::muxer::TsMuxer;
use ts_core::crc32::crc32_mpeg2;

#[test]
fn packet_builder_round_trip() {
    let payload_data = [0xAA; 184];
    let pkt_bytes = TsPacketBuilder::new(0x100, 5)
        .payload(&payload_data)
        .build();

    let pkt = TsPacket::parse(&pkt_bytes).unwrap();
    assert_eq!(pkt.header.pid, 0x100);
    assert_eq!(pkt.header.continuity_counter, 5);
    assert_eq!(pkt.payload.as_ref().unwrap().len(), 184);
    assert_eq!(pkt.payload.as_ref().unwrap()[0], 0xAA);
}

#[test]
fn packet_builder_with_pcr() {
    let pcr_val = 27_000_000u64; // 1 second
    let pkt_bytes = TsPacketBuilder::new(0x101, 0)
        .with_pcr(pcr_val)
        .payload(&[0xBB; 100])
        .build();

    let pkt = TsPacket::parse(&pkt_bytes).unwrap();
    assert_eq!(pkt.header.pid, 0x101);
    assert!(pkt.adaptation.is_some());
    let af = pkt.adaptation.as_ref().unwrap();
    assert_eq!(af.pcr.unwrap(), pcr_val);
}

#[test]
fn null_packet_is_valid() {
    let null = build_null_packet();
    let pkt = TsPacket::parse(&null).unwrap();
    assert_eq!(pkt.header.pid, 0x1FFF);
}

#[test]
fn pat_builder_round_trip() {
    let pat_section = PatBuilder::new(1)
        .add_program(1, 0x100)
        .add_program(2, 0x200)
        .build_section();

    let section = PsiSection::parse(&pat_section).unwrap();
    let pat = Pat::parse(&section).unwrap();
    assert_eq!(pat.transport_stream_id, 1);
    assert_eq!(pat.entries.len(), 2);
    assert_eq!(pat.entries[0].program_number, 1);
    assert_eq!(pat.entries[0].pid, 0x100);
    assert_eq!(pat.entries[1].program_number, 2);
    assert_eq!(pat.entries[1].pid, 0x200);
}

#[test]
fn pat_builder_crc_valid() {
    let section = PatBuilder::new(1)
        .add_program(1, 0x100)
        .build_section();

    // CRC covers everything except the last 4 bytes
    let crc = crc32_mpeg2(&section[..section.len() - 4]);
    let stored_crc = u32::from_be_bytes([
        section[section.len() - 4],
        section[section.len() - 3],
        section[section.len() - 2],
        section[section.len() - 1],
    ]);
    assert_eq!(crc, stored_crc);
}

#[test]
fn pmt_builder_round_trip() {
    let pmt_section = PmtBuilder::new(1, 0x101)
        .add_stream(0x1B, 0x101)
        .add_stream(0x0F, 0x102)
        .build_section();

    let section = PsiSection::parse(&pmt_section).unwrap();
    let pmt = Pmt::parse(&section).unwrap();
    assert_eq!(pmt.program_number, 1);
    assert_eq!(pmt.pcr_pid, 0x101);
    assert_eq!(pmt.streams.len(), 2);
    assert_eq!(pmt.streams[0].stream_type, 0x1B);
    assert_eq!(pmt.streams[0].elementary_pid, 0x101);
    assert_eq!(pmt.streams[1].stream_type, 0x0F);
    assert_eq!(pmt.streams[1].elementary_pid, 0x102);
}

#[test]
fn muxer_produces_valid_packets() {
    let mut muxer = TsMuxer::new(1, 0x100, 0x101, 5_000_000);
    muxer.add_stream(0x101, 0x1B); // video
    muxer.add_stream(0x102, 0x0F); // audio

    // push some data
    muxer.push_data(0x101, &[0xAA; 1000]);
    muxer.push_data(0x102, &[0xBB; 500]);

    let mut output = Vec::new();
    muxer.mux_packets(&mut output);

    assert!(!output.is_empty());

    // all packets must be valid 188-byte TS
    for pkt_bytes in &output {
        assert_eq!(pkt_bytes.len(), TS_PACKET_SIZE);
        let pkt = TsPacket::parse(pkt_bytes).unwrap();
        assert_eq!(pkt.header.sync_byte, 0x47);
    }

    // first two should be PAT(pid=0) and PMT(pid=0x100)
    let pkt0 = TsPacket::parse(&output[0]).unwrap();
    assert_eq!(pkt0.header.pid, 0x0000);
    let pkt1 = TsPacket::parse(&output[1]).unwrap();
    assert_eq!(pkt1.header.pid, 0x0100);
}

#[test]
fn muxer_cbr_stuffing() {
    let mut muxer = TsMuxer::new(1, 0x100, 0x101, 1_000_000);
    muxer.add_stream(0x101, 0x1B);
    muxer.push_data(0x101, &[0xAA; 100]);

    let mut output = Vec::new();
    muxer.mux_packets(&mut output);
    let data_packets = output.len();

    muxer.stuff_to_cbr(&mut output, 50);
    assert_eq!(output.len(), 50);

    // extra packets should be null
    for pkt_bytes in &output[data_packets..] {
        let pkt = TsPacket::parse(pkt_bytes).unwrap();
        assert_eq!(pkt.header.pid, 0x1FFF);
    }
}
