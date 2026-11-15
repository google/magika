// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// DO NOT EDIT, see link below for more information:
// https://github.com/google/magika/blob/main/scripts/sync_kb.py

use std::ptr;

use crate::MagikaTypeInfo;

#[rustfmt::skip] pub(crate) static _1C_ENTERPRISE_1CD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"1c_enterprise_1cd".as_ptr(),
    mime_type: c"application/x-1c-enterprise-db".as_ptr(),
    group: c"database".as_ptr(),
    description: c"1C:Enterprise 8 Infobase Database (1Cv8.1CD)".as_ptr(),
    extensions: [c"1cd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _1C_ENTERPRISE_CF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"1c_enterprise_cf".as_ptr(),
    mime_type: c"application/x-1c-enterprise-cf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"1C:Enterprise 8 Configuration Container (.cf)".as_ptr(),
    extensions: [c"cf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _2DKGT2_DEMO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"2dkgt2_demo".as_ptr(),
    mime_type: c"application/x-2dkgt2-demo".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"2D Fighter Maker 2nd (2DKGT2) Replay Demo (.demo, 2002) Enterbrain".as_ptr(),
    extensions: [c"demo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _3DS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"3ds".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Nintendo 3DS roms".as_ptr(),
    extensions: [c"3ds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _3DSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"3dsm".as_ptr(),
    mime_type: c"application/x-3ds".as_ptr(),
    group: c"image".as_ptr(),
    description: c"3D studio Max".as_ptr(),
    extensions: [c"3ds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _3DSX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"3dsx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Nintendo 3DS homebrew".as_ptr(),
    extensions: [c"3dsx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _3GP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"3gp".as_ptr(),
    mime_type: c"video/3gpp".as_ptr(),
    group: c"video".as_ptr(),
    description: c"3GPP multimedia file".as_ptr(),
    extensions: [c"3gp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static _3MF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"3mf".as_ptr(),
    mime_type: c"application/vnd.ms-package.3dmanufacturing-3dmodel+xml".as_ptr(),
    group: c"image".as_ptr(),
    description: c"3D Manufacturing Format".as_ptr(),
    extensions: [c"3mf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static A2R: MagikaTypeInfo = MagikaTypeInfo {
    label: c"a2r".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Applesauce A2R floppy flux capture".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static A78_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"a78_rom".as_ptr(),
    mime_type: c"application/x-atari-7800-rom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari 7800 ProSystem Cartridge ROM (.a78)".as_ptr(),
    extensions: [c"a78".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AAPT2_FLAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aapt2_flat".as_ptr(),
    mime_type: c"application/x-android-aapt2-flat".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Google Android AAPT2 Compiled Flat Resource Container (.flat)".as_ptr(),
    extensions: [c"flat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AARU_DICF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aaru_dicf".as_ptr(),
    mime_type: c"application/x-aaru-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Aaru Data Preservation Suite / DiscImageChef Media Image (.aaru / .dicf)".as_ptr(),
    extensions: [c"aaru".as_ptr(), c"dicf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ABCOMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"abcomp".as_ptr(),
    mime_type: c"application/x-abcomp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ABComp v2.x Compressed Data (.abp)".as_ptr(),
    extensions: [c"abp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ABLETON_AMXD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ableton_amxd".as_ptr(),
    mime_type: c"application/x-ableton-amxd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ableton Live / Cycling '74 Max for Live Device (.amxd, 2009) Ableton & Cycling '74".as_ptr(),
    extensions: [c"amxd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ABLETON_ASD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ableton_asd".as_ptr(),
    mime_type: c"application/x-ableton-asd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ableton Live Audio Analysis & Warp Sidecar (.asd)".as_ptr(),
    extensions: [c"asd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ABSOLUTE_DB_QC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"absolute_db_qc".as_ptr(),
    mime_type: c"application/x-absolute-db-qc".as_ptr(),
    group: c"database".as_ptr(),
    description: c"ComponentAce Absolute Database File (.abs / .qc)".as_ptr(),
    extensions: [c"abs".as_ptr(), c"qc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AC1D_PACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ac1d_packer".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Anarchy Amiga AC1D Packer Module (.ac1d)".as_ptr(),
    extensions: [c"ac1d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACCESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"access".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Access database".as_ptr(),
    extensions: [c"accdb".as_ptr(), c"mdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ace".as_ptr(),
    mime_type: c"application/x-ace-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ACE archive".as_ptr(),
    extensions: [c"ace".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACR5_ICDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"acr5_icdb".as_ptr(),
    mime_type: c"application/x-acr5-icdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"ACDSee ACR5 Image Catalog Database (.icdb, ACDSee)".as_ptr(),
    extensions: [c"icdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACRONIS_MSG_CATALOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"acronis_msg_catalog".as_ptr(),
    mime_type: c"application/x-acronis-msg-catalog".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Acronis Bootable Media Builder UTF-16LE Message String Catalog (.msg)".as_ptr(),
    extensions: [c"msg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACRONIS_TIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"acronis_tib".as_ptr(),
    mime_type: c"application/x-acronis-tib".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Acronis True Image backup container (.tib)".as_ptr(),
    extensions: [c"tib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACTA_OUTLINE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"acta_outline".as_ptr(),
    mime_type: c"application/x-acta-outline".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Symmetry Acta / Acta Advantage Macintosh Outline Document (.acta)".as_ptr(),
    extensions: [c"acta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ACTIONAMICS_AST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"actionamics_ast".as_ptr(),
    mime_type: c"audio/x-ast".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Actionamics Amiga Sound Tool Module (.ast)".as_ptr(),
    extensions: [c"ast".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADEXPLORER_DAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adexplorer_dat".as_ptr(),
    mime_type: c"application/x-adexplorer-dat".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Sysinternals Active Directory Explorer Snapshot (.dat)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_A2M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_a2m".as_ptr(),
    mime_type: c"audio/x-a2m".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Subz3ro AdLib Tracker II Module (.a2m)".as_ptr(),
    extensions: [c"a2m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_BAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_bam".as_ptr(),
    mime_type: c"audio/x-adlib-bam".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bob's AdLib Music Module (.bam)".as_ptr(),
    extensions: [c"bam".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_CMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_cmf".as_ptr(),
    mime_type: c"audio/x-cmf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Creative Labs AdLib / Sound Blaster Creative Music File (.cmf)".as_ptr(),
    extensions: [c"cmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_DRO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_dro".as_ptr(),
    mime_type: c"audio/x-dosbox-dro".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"DOSBox AdLib OPL2/OPL3 Raw Capture (.dro)".as_ptr(),
    extensions: [c"dro".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_FERNANDEZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_fernandez".as_ptr(),
    mime_type: c"audio/x-adlib-fernandez".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Martin Fernandez AdLib Music Module (.adlib)".as_ptr(),
    extensions: [c"adlib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_RAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_rad".as_ptr(),
    mime_type: c"audio/x-adlib-rad".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Reality AdLib Tracker OPL2/OPL3 Module (.rad)".as_ptr(),
    extensions: [c"rad".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_RAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_raw".as_ptr(),
    mime_type: c"audio/x-adlib-raw".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"RdosPlay AdLib OPL2 Raw Register Capture (.raw)".as_ptr(),
    extensions: [c"raw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_ROL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_rol".as_ptr(),
    mime_type: c"audio/x-rol".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"AdLib Visual Composer FM Song (.rol)".as_ptr(),
    extensions: [c"rol".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADLIB_SBI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adlib_sbi".as_ptr(),
    mime_type: c"audio/x-sbi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Creative Labs Sound Blaster FM Instrument Patch (.sbi)".as_ptr(),
    extensions: [c"sbi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADOBE_ASN_SIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adobe_asn_sif".as_ptr(),
    mime_type: c"application/x-adobe-asn-sif".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Adobe Developer Association (ASN) Search Index File (.sif, 1995) Adobe".as_ptr(),
    extensions: [c"sif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADOBE_RNA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adobe_rna".as_ptr(),
    mime_type: c"application/x-adobe-rna".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Adobe Acrobat Reader RNA IPC Message (.13ec / .7a4)".as_ptr(),
    extensions: [c"13ec".as_ptr(), c"7a4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADOBE_SWATCH_EXCHANGE_ASE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adobe_swatch_exchange_ase".as_ptr(),
    mime_type: c"application/x-adobe-swatch-exchange-ase".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Adobe Swatch Exchange (.ase, PRONOM fmt/1816)".as_ptr(),
    extensions: [c"ase".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADOBE_TECHPOOL_TUTORIAL_TPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adobe_techpool_tutorial_tpp".as_ptr(),
    mime_type: c"application/x-adobe-techpool-tutorial-tpp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Adobe / TechPool Interactive Tutorial Presentation (.tpp, 1995) TechPool Studios / Adobe".as_ptr(),
    extensions: [c"tpp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADRIFT_TAF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"adrift_taf".as_ptr(),
    mime_type: c"application/x-adrift-taf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"ADRIFT Text Adventure File (.taf, PRONOM fmt/1584)".as_ptr(),
    extensions: [c"taf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADVANCED_FORENSIC_FORMAT_AFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"advanced_forensic_format_aff".as_ptr(),
    mime_type: c"application/x-advanced-forensic-format-aff".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Advanced Forensic Format (.aff, PRONOM fmt/844)".as_ptr(),
    extensions: [c"aff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ADVSYS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"advsys".as_ptr(),
    mime_type: c"application/x-advsys".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"AdvSys Compiled Adventure (.dat)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AEROFLY_TMT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aerofly_tmt".as_ptr(),
    mime_type: c"image/x-aerofly-tmt".as_ptr(),
    group: c"image".as_ptr(),
    description: c"IPACS Aerofly Texture Map (.tmt)".as_ptr(),
    extensions: [c"tmt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aff".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Hunspell Affix".as_ptr(),
    extensions: [c"aff".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static AGS_TRA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ags_tra".as_ptr(),
    mime_type: c"application/x-ags-tra".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Adventure Game Studio Compiled Translation (.tra, 2001) Chris Jones".as_ptr(),
    extensions: [c"tra".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AGT_AGX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"agt_agx".as_ptr(),
    mime_type: c"application/x-agt-agx".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"AGT Adventure Game Toolkit AGX Game (.agx)".as_ptr(),
    extensions: [c"agx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ai".as_ptr(),
    mime_type: c"application/pdf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Adobe Illustrator Artwork".as_ptr(),
    extensions: [c"ai".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AIDAIM_SFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aidaim_sfs".as_ptr(),
    mime_type: c"application/x-aidaim-sfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"AidAim Software SingleFileSystem (SFS) Container (AAMVHFSS, .edb / .plr / .loader)".as_ptr(),
    extensions: [c"edb".as_ptr(), c"plr".as_ptr(), c"loader".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AIDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aidl".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Android Interface Definition Language".as_ptr(),
    extensions: [c"aidl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ALAN3_GAME: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alan3_game".as_ptr(),
    mime_type: c"application/x-alan-adventure".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Alan 3 Compiled Adventure (.a3c)".as_ptr(),
    extensions: [c"a3c".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALCH_GAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alch_gal".as_ptr(),
    mime_type: c"image/x-alchemy-gallery".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Alchemy Mindworks Image Gallery (.gal)".as_ptr(),
    extensions: [c"gal".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALEMBIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alembic".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Alembic geometry cache".as_ptr(),
    extensions: [c"abc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALL_SOUND_TRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"all_sound_tracker".as_ptr(),
    mime_type: c"audio/x-ast".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"All Sound Tracker Module (.ast)".as_ptr(),
    extensions: [c"ast".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALLEGRO_FASL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"allegro_fasl".as_ptr(),
    mime_type: c"application/x-allegro-fasl".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Franz Allegro Common Lisp Fast-Load Bytecode (.fasl, 1988) Franz".as_ptr(),
    extensions: [c"fasl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALLWINNER_EGON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"allwinner_egon".as_ptr(),
    mime_type: c"application/x-allwinner-egon".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Allwinner eGON.BT0 / eGON.BT1 Bootloader Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALLWINNER_IMAGEWTY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"allwinner_imagewty".as_ptr(),
    mime_type: c"application/x-allwinner-imagewty".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Allwinner LiveSuit / PhoenixSuit IMAGEWTY Firmware Image (.img)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALPHA_BMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alpha_bmp".as_ptr(),
    mime_type: c"image/x-alpha-bmp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Alpha Microsystems AMOS Bitmap (.bmp)".as_ptr(),
    extensions: [c"bmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALPHA_WRGG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alpha_wrgg".as_ptr(),
    mime_type: c"application/x-alpha-wrgg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Alpha Networks WRGG03 / DKBS Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALSA_SOC_TOPOLOGY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alsa_soc_topology".as_ptr(),
    mime_type: c"application/x-alsa-topology".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Linux ALSA SoC Audio DSP Topology Binary (.tplg, CoSA)".as_ptr(),
    extensions: [c"tplg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALTO_AL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alto_al".as_ptr(),
    mime_type: c"application/x-alto-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Xerox Alto Bitmap Font (.al, 1974) Xerox PARC".as_ptr(),
    extensions: [c"al".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALTO_DSK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alto_dsk".as_ptr(),
    mime_type: c"application/x-xerox-alto-disk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Xerox Alto Diablo 31 Disk Pack Image (.dsk, 1973) Xerox PARC".as_ptr(),
    extensions: [c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALTO_RUN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alto_run".as_ptr(),
    mime_type: c"application/x-xerox-alto-run".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Xerox Alto BCPL Save Version Executable (.run, 1974) Xerox PARC".as_ptr(),
    extensions: [c"run".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALTO_WIDTHS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alto_widths".as_ptr(),
    mime_type: c"application/x-alto-widths".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Xerox Alto Font Widths Table (FONTS.WIDTHS, 1975) Xerox PARC".as_ptr(),
    extensions: [c"widths".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ALZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"alz".as_ptr(),
    mime_type: c"application/x-alzip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ESTsoft ALZip compressed archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AM6_AMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"am6_amp".as_ptr(),
    mime_type: c"application/x-am6-amp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"AM6 Wrapped Windows DIB Bitmap (.amp)".as_ptr(),
    extensions: [c"amp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AM_ACETRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"am_acetracker".as_ptr(),
    mime_type: c"audio/x-am".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga Ace Tracker Module (.am)".as_ptr(),
    extensions: [c"am".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AM_COMPOSER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"am_composer".as_ptr(),
    mime_type: c"audio/x-am-composer".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga A.M. Composer 1.2 Module (.amc)".as_ptr(),
    extensions: [c"amc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMD_MICROCODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amd_microcode".as_ptr(),
    mime_type: c"application/x-amd-microcode".as_ptr(),
    group: c"application".as_ptr(),
    description: c"AMD x86-64 Processor Microcode Update Container (.bin, DMA\\\\0)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMD_PSP_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amd_psp_firmware".as_ptr(),
    mime_type: c"application/x-amd-psp-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"AMD Platform Security Processor / NPU / TEE Signed Firmware ($PS1)".as_ptr(),
    extensions: [c"sbin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMDGPU_UCODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amdgpu_ucode".as_ptr(),
    mime_type: c"application/x-amdgpu-microcode".as_ptr(),
    group: c"application".as_ptr(),
    description: c"AMD Radeon / AMDGPU Graphics & Compute Microcode Binary (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMF_DSMI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amf_dsmi".as_ptr(),
    mime_type: c"audio/x-amf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Otto Chrons DSMI Advanced Module Format (.amf)".as_ptr(),
    extensions: [c"amf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMG_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amg_archive".as_ptr(),
    mime_type: c"application/x-amg-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Milen Georgiev AMG Compressed Archive (.amg)".as_ptr(),
    extensions: [c"amg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_AHX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_ahx".as_ptr(),
    mime_type: c"audio/x-ahx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga Abyss Highest Experience Chiptune Synthesizer Module (.ahx / .thx)".as_ptr(),
    extensions: [c"ahx".as_ptr(), c"thx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_CDTV_RESIDENT_TM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_cdtv_resident_tm".as_ptr(),
    mime_type: c"application/x-amiga-cdtv-resident-tm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Commodore Amiga CDTV Resident Trademark / Device Module (CDTV.TM, 1991) Commodore-Amiga".as_ptr(),
    extensions: [c"tm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_DBM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_dbm".as_ptr(),
    mime_type: c"audio/x-dbm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga DigiBooster / DigiBooster Pro Tracker Module (.dbm / .digi)".as_ptr(),
    extensions: [c"dbm".as_ptr(), c"digi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_FC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_fc".as_ptr(),
    mime_type: c"audio/x-future-composer".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga Future Composer v1.0-v1.4 Synth Module (.fc / .fc13 / .fc14 / .smod)".as_ptr(),
    extensions: [c"fc".as_ptr(), c"fc13".as_ptr(), c"fc14".as_ptr(), c"smod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_OKT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_okt".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga Oktalyzer 8-Channel Tracker Module (.okt)".as_ptr(),
    extensions: [c"okt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_PACKDEV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_packdev".as_ptr(),
    mime_type: c"application/x-amiga-packdev".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Amiga PackDev Device Archive (.pkd / .dsq)".as_ptr(),
    extensions: [c"pkd".as_ptr(), c"dsq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_SQUASH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_squash".as_ptr(),
    mime_type: c"application/x-amiga-squash".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Squash Amiga Data Compressor (.sqsh)".as_ptr(),
    extensions: [c"sqsh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMIGA_ZOOM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amiga_zoom".as_ptr(),
    mime_type: c"application/x-amiga-zoom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Amiga Zoom Floppy Disk Archive (.zom)".as_ptr(),
    extensions: [c"zom".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMLOGIC_MPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amlogic_mpt".as_ptr(),
    mime_type: c"application/x-amlogic-mpt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Amlogic SoC eMMC Partition Table (MPT\\0)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMLOGIC_UPGRADE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amlogic_upgrade".as_ptr(),
    mime_type: c"application/x-amlogic-upgrade".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Amlogic USB Burning Tool Upgrade Package (aml_upgrade_package.img) Amlogic".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMPLESOUND_UMSK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amplesound_umsk".as_ptr(),
    mime_type: c"application/x-amplesound-umsk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ample Sound Guitar/Bass Keyswitch Articulation Map (.umsk) Ample Sound".as_ptr(),
    extensions: [c"umsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMS_EXTREME: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ams_extreme".as_ptr(),
    mime_type: c"audio/x-ams".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Velvet Development Extreme's Tracker Module (.ams)".as_ptr(),
    extensions: [c"ams".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AMX_SYNTH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"amx_synth".as_ptr(),
    mime_type: c"audio/x-amx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"And X Synth Module (.amx)".as_ptr(),
    extensions: [c"amx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANC_CRUNCHER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"anc_cruncher".as_ptr(),
    mime_type: c"application/x-anc-cruncher".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ANC Cruncher Amiga Data Stream (.anc)".as_ptr(),
    extensions: [c"anc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDERS_OLAND: MagikaTypeInfo = MagikaTypeInfo {
    label: c"anders_oland".as_ptr(),
    mime_type: c"audio/x-anders-oland".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Anders 0land Amiga Music Module (.hot)".as_ptr(),
    extensions: [c"hot".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_ART_PROFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_art_profile".as_ptr(),
    mime_type: c"application/vnd.android.prof".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Android Runtime (ART) Baseline Profile (.prof / .profm)".as_ptr(),
    extensions: [c"prof".as_ptr(), c"profm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_BOOTLOADER_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_bootloader_img".as_ptr(),
    mime_type: c"application/x-android-bootloader".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Android Bootloader Signed Image Wrapper (BOOTAND)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_DTBO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_dtbo".as_ptr(),
    mime_type: c"application/x-android-dtbo".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Android Device Tree Blob Overlay Partition Image (.dtbo, 0x1EABB7D7)".as_ptr(),
    extensions: [c"dtbo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_HYB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_hyb".as_ptr(),
    mime_type: c"application/x-android-hyb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Android Minikin Compiled Hyphenation Dictionary (.hyb)".as_ptr(),
    extensions: [c"hyb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_OTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_ota".as_ptr(),
    mime_type: c"application/x-android-ota-payload".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Google ChromeOS / Android A/B OTA Update Payload (CrAU)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_SPARSE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_sparse".as_ptr(),
    mime_type: c"application/x-android-sparse".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Android Sparse Ext4/EROFS System Partition Image (.simg)".as_ptr(),
    extensions: [c"simg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_VBMETA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_vbmeta".as_ptr(),
    mime_type: c"application/x-android-vbmeta".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google Android Verified Boot 2.0 VBMeta Image (AVB0)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANDROID_WINSCOPE_TRACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"android_winscope_trace".as_ptr(),
    mime_type: c"application/x-android-winscope-trace".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Android AOSP Winscope WindowManager / SurfaceFlinger / ProtoLog Trace".as_ptr(),
    extensions: [c"winscope".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ani".as_ptr(),
    mime_type: c"application/x-navi-animation".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Animated cursor".as_ptr(),
    extensions: [c"ani".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANSI_CSI_ART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ansi_csi_art".as_ptr(),
    mime_type: c"application/x-ansi-csi-art".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ANSI X3.64 / ECMA-48 CSI Escape BBS Text Art (.ans / .imp)".as_ptr(),
    extensions: [c"ans".as_ptr(), c"imp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ANSI_TAPE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ansi_tape".as_ptr(),
    mime_type: c"application/x-ansi-tape".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ANSI X3.27 / ISO 1001 Labeled Magnetic Tape Volume (VOL1/HDR1 ASCII labels, 1969) ANSI".as_ptr(),
    extensions: [c"ct".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AOE_DRS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aoe_drs".as_ptr(),
    mime_type: c"application/x-ensemble-drs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ensemble Studios Age of Empires DRS Resource Archive (.drs, 1997) Ensemble Studios".as_ptr(),
    extensions: [c"drs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AOL_ART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aol_art".as_ptr(),
    mime_type: c"image/x-jg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"America Online Johnson-Grace ART Compressed Image (.art)".as_ptr(),
    extensions: [c"art".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AON_FM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aon_fm".as_ptr(),
    mime_type: c"audio/x-aon-fm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bastian Spiegel Art Of Noise FM Synth Instrument (.fm)".as_ptr(),
    extensions: [c"fm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AOR_DV10_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aor_dv10_fw".as_ptr(),
    mime_type: c"application/x-aor-dv10-fw".as_ptr(),
    group: c"application".as_ptr(),
    description: c"AOR AR-DV10 SDR Receiver Encrypted Firmware Image (.d10) AOR".as_ptr(),
    extensions: [c"d10".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AOUT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aout".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"a.out object / executable".as_ptr(),
    extensions: [c"out".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APIS_IQ_ICE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apis_iq_ice".as_ptr(),
    mime_type: c"application/x-apis-iq-ice".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"APIS IQ-Software (IQ-FMEA / IQ-RM) Fridge Container (.ice / .cfg)".as_ptr(),
    extensions: [c"ice".as_ptr(), c"cfg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apk".as_ptr(),
    mime_type: c"application/vnd.android.package-archive".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Android package".as_ptr(),
    extensions: [c"apk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APOLLO_SYSBOOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apollo_sysboot".as_ptr(),
    mime_type: c"application/x-apollo-sysboot".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Apollo Domain/OS SYSBOOT Boot Tape Image (.ct / .img, 1985) Apollo Computer".as_ptr(),
    extensions: [c"ct".as_ptr(), c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_8900: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_8900".as_ptr(),
    mime_type: c"application/x-apple-8900".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple S5L8900 Secure Boot Firmware Container (89001.0, .dfu / .img2) Apple".as_ptr(),
    extensions: [c"dfu".as_ptr(), c"img2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_AEA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_aea".as_ptr(),
    mime_type: c"application/x-apple-aea".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple Encrypted Archive / Signed Shortcut (.aea / .shortcut, 2021) Apple".as_ptr(),
    extensions: [c"aea".as_ptr(), c"shortcut".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_archive".as_ptr(),
    mime_type: c"application/x-apple-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple Archive (.aar / .yaa)".as_ptr(),
    extensions: [c"aar".as_ptr(), c"yaa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_ASL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_asl".as_ptr(),
    mime_type: c"application/x-apple-asl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple System Log Binary Database (.asl)".as_ptr(),
    extensions: [c"asl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_BINARY_II: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_binary_ii".as_ptr(),
    mime_type: c"application/x-apple-binary-ii".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple II Binary II Archive (.bny / .bqy)".as_ptr(),
    extensions: [c"bny".as_ptr(), c"bqy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_FAIRPLAY_SUPF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_fairplay_supf".as_ptr(),
    mime_type: c"application/x-apple-fairplay-supf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple FairPlay DRM f-Class Key Container (.supf)".as_ptr(),
    extensions: [c"supf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_FAIRPLAY_SUPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_fairplay_supp".as_ptr(),
    mime_type: c"application/x-apple-fairplay-supp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple FairPlay DRM p-Class Key Container (.supp)".as_ptr(),
    extensions: [c"supp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_MKEXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_mkext".as_ptr(),
    mime_type: c"application/x-apple-mkext".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple Darwin Multi-Kext Kernel Extension Archive (.mkext, 2000) Apple".as_ptr(),
    extensions: [c"mkext".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_NIBARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_nibarchive".as_ptr(),
    mime_type: c"application/x-apple-nibarchive".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Apple Compiled Interface Builder NIBArchive (.nib)".as_ptr(),
    extensions: [c"nib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_SECURITYD_BLOB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_securityd_blob".as_ptr(),
    mime_type: c"application/x-apple-securityd-blob".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple SecurityServer CommonBlob Key/Database Store (.db, 2000) Apple".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_SPARSEIMAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_sparseimage".as_ptr(),
    mime_type: c"application/x-apple-sparseimage".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple macOS Sparse Disk Image (.sparseimage, sprs)".as_ptr(),
    extensions: [c"sparseimage".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_SPOTLIGHT_STORE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_spotlight_store".as_ptr(),
    mime_type: c"application/x-apple-spotlight-store".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Apple Spotlight Metadata Store Database (store.db)".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLE_VMP4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"apple_vmp4".as_ptr(),
    mime_type: c"application/x-apple-vmp4".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple Maps Vector Map Tile Container (VMP4, .vf) Apple".as_ptr(),
    extensions: [c"vf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLEBPLIST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"applebplist".as_ptr(),
    mime_type: c"application/x-bplist".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple binary property list".as_ptr(),
    extensions: [c"bplist".as_ptr(), c"plist".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLEDOUBLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"appledouble".as_ptr(),
    mime_type: c"multipart/appledouble".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"AppleDouble".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APPLEPLIST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"appleplist".as_ptr(),
    mime_type: c"application/x-plist".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple property list".as_ptr(),
    extensions: [c"plist".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static APPLESINGLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"applesingle".as_ptr(),
    mime_type: c"application/applefile".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"AppleSingle".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APROSYS_APS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aprosys_aps".as_ptr(),
    mime_type: c"audio/x-aprosys".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Petter Urstad AProSys Amiga Synth Module (.aps)".as_ptr(),
    extensions: [c"aps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static APS_N64_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aps_n64_patch".as_ptr(),
    mime_type: c"application/x-aps-patch".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nintendo 64 APS 1.0 ROM Patch (.aps, APS10)".as_ptr(),
    extensions: [c"aps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ar".as_ptr(),
    mime_type: c"application/x-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"AR Archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arc".as_ptr(),
    mime_type: c"application/x-arc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Arc".as_ptr(),
    extensions: [c"arc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARC_CASTPART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arc_castpart".as_ptr(),
    mime_type: c"application/x-arc-castpart".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ARC CAST Airport Simulation Component (CT COBJ2, .CASTPart)".as_ptr(),
    extensions: [c"castpart".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARCADYAN_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arcadyan_fw".as_ptr(),
    mime_type: c"application/x-arcadyan-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Arcadyan Obfuscated LZMA Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARCGRID: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arcgrid".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ESRI ASCII Grid".as_ptr(),
    extensions: [c"asc".as_ptr(), c"grd".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ARCHIVE_HASH_RTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"archive_hash_rtm".as_ptr(),
    mime_type: c"application/x-archive-hash-rtm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ARCHIVE# v2 Resource Package (.rtm)".as_ptr(),
    extensions: [c"rtm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARCSOFT_UI_AUI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arcsoft_ui_aui".as_ptr(),
    mime_type: c"application/x-arcsoft-ui-aui".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ArcSoft PhotoStudio UI Layout Definition (.aui, 1998) ArcSoft".as_ptr(),
    extensions: [c"aui".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARCV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arcv".as_ptr(),
    mime_type: c"application/x-arcv".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Eschalon Setup ARCV Compressed Container (.arv)".as_ptr(),
    extensions: [c"arv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARGONAUT_ASF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"argonaut_asf".as_ptr(),
    mime_type: c"audio/x-argonaut-asf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Argonaut Games ASF ADPCM Audio Stream (.asf)".as_ptr(),
    extensions: [c"asf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARGONAUT_BRP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"argonaut_brp".as_ptr(),
    mime_type: c"video/x-argonaut-brp".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Argonaut Games BRPP Full-Motion Video (.brp)".as_ptr(),
    extensions: [c"brp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arj".as_ptr(),
    mime_type: c"application/arj".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Arj".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARM_MALI_CSFFW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arm_mali_csffw".as_ptr(),
    mime_type: c"application/x-arm-mali-csffw".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ARM Mali GPU Command Stream Frontend MCU Firmware (mali_csffw.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARM_VECTOR_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arm_vector_rom".as_ptr(),
    mime_type: c"application/x-arm-vector-rom".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"ARM32 Exception Vector Firmware ROM Image (.rom)".as_ptr(),
    extensions: [c"rom".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ARROW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"arrow".as_ptr(),
    mime_type: c"application/vnd.apache.arrow.file".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"arrow".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"art".as_ptr(),
    mime_type: c"application/vnd.android.art".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Android Runtime Image (ART)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ART_OF_NOISE_AON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"art_of_noise_aon".as_ptr(),
    mime_type: c"audio/x-aon".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bastian Spiegel Art Of Noise 4/8-Channel Amiga Tracker (.aon, AON4/AON8)".as_ptr(),
    extensions: [c"aon".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asd".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"asd".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asdf".as_ptr(),
    mime_type: c"application/x-asdf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Advanced Scientific Data Format (ASDF, YAML + binary blocks)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asf".as_ptr(),
    mime_type: c"video/x-ms-asf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Advanced Systems Format".as_ptr(),
    extensions: [c"asf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASHAMPOO_ASHLANG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ashampoo_ashlang".as_ptr(),
    mime_type: c"application/x-ashampoo-ashlang".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ashampoo URESFILE Localization Package (.ashlang) Ashampoo".as_ptr(),
    extensions: [c"ashlang".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASKSAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asksam".as_ptr(),
    mime_type: c"application/x-asksam".as_ptr(),
    group: c"database".as_ptr(),
    description: c"askSam Free-Form Database (.ask, PRONOM fmt/2019)".as_ptr(),
    extensions: [c"ask".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asm".as_ptr(),
    mime_type: c"text/x-asm".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Assembly".as_ptr(),
    extensions: [c"s".as_ptr(), c"S".as_ptr(), c"asm".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ASP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asp".as_ptr(),
    mime_type: c"text/html".as_ptr(),
    group: c"code".as_ptr(),
    description: c"ASP source".as_ptr(),
    extensions: [c"aspx".as_ptr(), c"asp".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ASSETTO_CORSA_KN5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"assetto_corsa_kn5".as_ptr(),
    mime_type: c"model/x-assetto-corsa-kn5".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Kunos Simulazioni Assetto Corsa KN5 3D Model (.kn5)".as_ptr(),
    extensions: [c"kn5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASSETTOCORSA_ACD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"assettocorsa_acd".as_ptr(),
    mime_type: c"application/x-assettocorsa-acd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Kunos Simulazioni Assetto Corsa Encrypted Car Data Archive (.acd, 2013) Kunos Simulazioni".as_ptr(),
    extensions: [c"acd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASTC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"astc".as_ptr(),
    mime_type: c"image/astc".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ARM Adaptive Scalable Texture Compression Image (.astc)".as_ptr(),
    extensions: [c"astc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASTM_E57: MagikaTypeInfo = MagikaTypeInfo {
    label: c"astm_e57".as_ptr(),
    mime_type: c"model/e57".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ASTM E57 3D Imaging Lidar Point Cloud (.e57, ASTM-E57)".as_ptr(),
    extensions: [c"e57".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASYLUM_MUSIC_FORMAT_AMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asylum_music_format_amf".as_ptr(),
    mime_type: c"application/x-asylum-music-format-amf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Asylum Music Format (.amf, PRONOM fmt/1622)".as_ptr(),
    extensions: [c"amf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ASZ_PACKAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"asz_package".as_ptr(),
    mime_type: c"application/x-asz-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ASzf Zlib-Compressed Resource Package (.asz)".as_ptr(),
    extensions: [c"asz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_CAD3D2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_cad3d2".as_ptr(),
    mime_type: c"model/x-atari-cad3d2".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Antic Software Atari ST Cyber Studio CAD-3D 2.0 3D Object (.3d2, 1987) Tom Hudson / Antic Publishing".as_ptr(),
    extensions: [c"3d2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_CRACKART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_crackart".as_ptr(),
    mime_type: c"image/x-crackart".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Atari ST CrackArt Raster Image (.ca1 / .ca2 / .ca3)".as_ptr(),
    extensions: [c"ca1".as_ptr(), c"ca2".as_ptr(), c"ca3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_GFA_BASIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_gfa_basic".as_ptr(),
    mime_type: c"application/x-gfa-basic".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"GFA Systemtechnik Atari ST GFA-BASIC 3.x Program (.gfa, 1988) Frank Ostrowski / GFA Systemtechnik".as_ptr(),
    extensions: [c"gfa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_MPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_mpp".as_ptr(),
    mime_type: c"image/x-atari-mpp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Atari ST Multi Palette Picture File (.mpp, PRONOM fmt/1471)".as_ptr(),
    extensions: [c"mpp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_MSA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_msa".as_ptr(),
    mime_type: c"application/x-atari-msa".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST Magic Shadow Archiver Compressed Floppy Image (.msa)".as_ptr(),
    extensions: [c"msa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_SAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_sap".as_ptr(),
    mime_type: c"audio/x-sap".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Atari 8-Bit Slight Atari Player POKEY Chiptune (.sap)".as_ptr(),
    extensions: [c"sap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_ST_GUIDE_HYP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_st_guide_hyp".as_ptr(),
    mime_type: c"application/x-atari-st-guide".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Atari ST-Guide Compiled Hypertext (.hyp, HDOC)".as_ptr(),
    extensions: [c"hyp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_STX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_stx".as_ptr(),
    mime_type: c"application/x-atari-stx".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST Pasti Copy-Protected Floppy Disk Image (.stx)".as_ptr(),
    extensions: [c"stx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATARI_YM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atari_ym".as_ptr(),
    mime_type: c"audio/x-ym".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Atari ST / Amstrad CPC YM2149 Sound Chip Music (.ym)".as_ptr(),
    extensions: [c"ym".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATLUS_GFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atlus_gfs".as_ptr(),
    mime_type: c"model/x-atlus-gfs".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Atlus GFS0 3D Model / Texture Container (.gmd / .gfs, 2016) Atlus / Sega".as_ptr(),
    extensions: [c"gmd".as_ptr(), c"gfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ATOMIK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"atomik".as_ptr(),
    mime_type: c"application/x-atomik".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST Atomik Cruncher v3.5 Compressed File (ATM5)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"au".as_ptr(),
    mime_type: c"audio/basic".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NeXT/Sun AU".as_ptr(),
    extensions: [c"au".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUDFPRINT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"audfprint".as_ptr(),
    mime_type: c"application/x-audfprint".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Audfprint Audio Landmark Fingerprint Database (.afpt / .afpk)".as_ptr(),
    extensions: [c"afpt".as_ptr(), c"afpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUDIBLE_AA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"audible_aa".as_ptr(),
    mime_type: c"audio/x-pn-audibleaudio".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Audible Legacy DRM Audiobook Container (.aa)".as_ptr(),
    extensions: [c"aa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUDIO_SCULPTURE_AS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"audio_sculpture_as".as_ptr(),
    mime_type: c"audio/x-audio-sculpture".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sync Audio Sculpture Synth Instrument Bank (.as)".as_ptr(),
    extensions: [c"as".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUL_DSC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aul_dsc".as_ptr(),
    mime_type: c"application/x-apple-aul-dsc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple Unified Logging Shared-Cache Strings (.dsc)".as_ptr(),
    extensions: [c"dsc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUL_TIMESYNC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aul_timesync".as_ptr(),
    mime_type: c"application/x-apple-aul-timesync".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple Unified Logging Timesync Database (.timesync)".as_ptr(),
    extensions: [c"timesync".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUL_UUIDTEXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"aul_uuidtext".as_ptr(),
    mime_type: c"application/x-apple-aul-uuidtext".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple Unified Logging UUIDText String Catalog".as_ptr(),
    extensions: [c"uuidtext".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AURORADSP_STATE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"auroradsp_state".as_ptr(),
    mime_type: c"application/x-auroradsp-state".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Aurora DSP Audio Plugin State Preset (.afx / .lpa) Aurora DSP".as_ptr(),
    extensions: [c"afx".as_ptr(), c"lpa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUTEL_ECC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autel_ecc".as_ptr(),
    mime_type: c"application/x-autel-ecc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Autel Diagnostic Encrypted Firmware Container (.ecc)".as_ptr(),
    extensions: [c"ecc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUTOCAD_VLX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autocad_vlx".as_ptr(),
    mime_type: c"application/x-autocad-vlx".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Autodesk AutoCAD Visual LISP Compiled Application Package (.vlx, 1999) Autodesk".as_ptr(),
    extensions: [c"vlx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUTOHOTKEY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autohotkey".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"AutoHotKey script".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static AUTOIT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autoit".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"AutoIt script".as_ptr(),
    extensions: [c"au3".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static AUTOLISP_FAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autolisp_fas".as_ptr(),
    mime_type: c"application/x-autolisp-fas".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Autodesk AutoCAD Visual LISP Compiled Fast-Load Bytecode (.fas, 1999) Autodesk".as_ptr(),
    extensions: [c"fas".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AUTORUN_MAESTRO_MENU_FILE_MNU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"autorun_maestro_menu_file_mnu".as_ptr(),
    mime_type: c"application/x-autorun-maestro-menu-file-mnu".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Longtion AutoRun Pro / Autorun Maestro Menu (.mnu, PRONOM fmt/1461)".as_ptr(),
    extensions: [c"mnu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AVAST_VPU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"avast_vpu".as_ptr(),
    mime_type: c"application/x-avast-vpu".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Avast Antivirus Setup & Definitions Package (.vpu / .vpx)".as_ptr(),
    extensions: [c"vpu".as_ptr(), c"vpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AVCHD_THUMBNAIL_INDEX_FILE_TID: MagikaTypeInfo = MagikaTypeInfo {
    label: c"avchd_thumbnail_index_file_tid".as_ptr(),
    mime_type: c"application/x-avchd-thumbnail-index-file-tid".as_ptr(),
    group: c"document".as_ptr(),
    description: c"AVCHD Thumbnail Index File (.tid, PRONOM fmt/1077)".as_ptr(),
    extensions: [c"tid".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AVI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"avi".as_ptr(),
    mime_type: c"video/x-msvideo".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Audio Video Interleave".as_ptr(),
    extensions: [c"avi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AVIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"avif".as_ptr(),
    mime_type: c"image/avif".as_ptr(),
    group: c"image".as_ptr(),
    description: c"AV1 Image File Format".as_ptr(),
    extensions: [c"avif".as_ptr(), c"avifs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AVR_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"avr_audio".as_ptr(),
    mime_type: c"audio/x-avr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Atari ST Audio Visual Research Sampled Sound (.avr, 2BIT)".as_ptr(),
    extensions: [c"avr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AVRO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"avro".as_ptr(),
    mime_type: c"application/x-avro-binary".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Apache Avro binary".as_ptr(),
    extensions: [c"avro".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AWARD_BIOS_STF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"award_bios_stf".as_ptr(),
    mime_type: c"application/x-award-bios-stf".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Award Software BIOS Setup Language Module (_XX_CODE.BIN, .stf)".as_ptr(),
    extensions: [c"stf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static AWK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"awk".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Awk".as_ptr(),
    extensions: [c"awk".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BAHN_NT3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bahn_nt3".as_ptr(),
    mime_type: c"application/x-bahn-nt3".as_ptr(),
    group: c"document".as_ptr(),
    description: c"JB Software BAHN 3.2 Railway Simulation Network Layout (.nt3, 1995) Jan Bochmann".as_ptr(),
    extensions: [c"nt3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BAI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bai".as_ptr(),
    mime_type: c"application/x-bai".as_ptr(),
    group: c"database".as_ptr(),
    description: c"SAMtools BAM Index (.bai)".as_ptr(),
    extensions: [c"bai".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bam".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"BAM alignment".as_ptr(),
    extensions: [c"bam".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BANDAI_NAMCO_NUS3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bandai_namco_nus3".as_ptr(),
    mime_type: c"audio/x-bandai-namco-nus3".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bandai Namco NUS3 Audio Container (.nus3audio / .nus3bank)".as_ptr(),
    extensions: [c"nus3audio".as_ptr(), c"nus3bank".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BANNERMANIA_FNT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bannermania_fnt".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Broderbund BannerMania Vector Font (.fnt)".as_ptr(),
    extensions: [c"fnt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BARTENDER_BTW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bartender_btw".as_ptr(),
    mime_type: c"application/x-bartender-btw".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Seagull Scientific BarTender Barcode Label Document (.btw, 1992) Seagull Scientific".as_ptr(),
    extensions: [c"btw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BASIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"basis".as_ptr(),
    mime_type: c"image/x-basis".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Binomial Basis Universal Supercompressed GPU Texture (.basis)".as_ptr(),
    extensions: [c"basis".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"batch".as_ptr(),
    mime_type: c"text/x-msdos-batch".as_ptr(),
    group: c"code".as_ptr(),
    description: c"DOS batch file".as_ptr(),
    extensions: [c"bat".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BAZEL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bazel".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Bazel build file".as_ptr(),
    extensions: [c"bzl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BBSONG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bbsong".as_ptr(),
    mime_type: c"audio/x-bbsong".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Beepola Beeper Song (.bbsong)".as_ptr(),
    extensions: [c"bbsong".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BCL_BFC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bcl_bfc".as_ptr(),
    mime_type: c"application/x-bcl-bfc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Marcus Geelnard Basic Compression Library File (.bfc)".as_ptr(),
    extensions: [c"bfc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BCRES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bcres".as_ptr(),
    mime_type: c"application/x-nintendo-bcres".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo 3DS NW4C Binary CTR Resource Package (.bcres / .bcmdl / .bctex)".as_ptr(),
    extensions: [c"bcres".as_ptr(), c"bcmdl".as_ptr(), c"bctex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BEAGLEWORKS_DOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"beagleworks_doc".as_ptr(),
    mime_type: c"application/x-beagleworks".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Beagle Bros BeagleWorks / WordPerfect Works Integrated Document (.bw / .wpw, BWks)".as_ptr(),
    extensions: [c"bw".as_ptr(), c"wpw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BEAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"beam".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Erlang BEAM".as_ptr(),
    extensions: [c"beam".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BEAVER_SWEEPER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"beaver_sweeper".as_ptr(),
    mime_type: c"audio/x-beaver-sweeper".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Beaver Sweeper Tracker Module (.gtk)".as_ptr(),
    extensions: [c"gtk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BERKELEY_VFONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"berkeley_vfont".as_ptr(),
    mime_type: c"font/x-berkeley-vfont".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Berkeley Unix / SunOS vfont Bitmap Font (.vfont, 1979) UC Berkeley CSRG / Sun Microsystems".as_ptr(),
    extensions: [c"vfont".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BERKELEYDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"berkeleydb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Berkeley DB".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BEROTRACKER_BRT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"berotracker_brt".as_ptr(),
    mime_type: c"audio/x-brt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Benjamin Rosseaux BeRoTracker Module (.brt)".as_ptr(),
    extensions: [c"brt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BETHESDA_BGSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bethesda_bgsm".as_ptr(),
    mime_type: c"application/x-bethesda-bgsm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Bethesda Creation Engine Shader Material (.bgsm / .bgem)".as_ptr(),
    extensions: [c"bgsm".as_ptr(), c"bgem".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BETHESDA_ESP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bethesda_esp".as_ptr(),
    mime_type: c"application/x-bethesda-esp".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Bethesda Creation Engine Plugin / Master (.esp / .esm / .esl)".as_ptr(),
    extensions: [c"esp".as_ptr(), c"esm".as_ptr(), c"esl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BETHESDA_FUZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bethesda_fuz".as_ptr(),
    mime_type: c"audio/x-bethesda-fuz".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bethesda Creation Engine Voice & Lip-Sync Audio (.fuz)".as_ptr(),
    extensions: [c"fuz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BETHESDA_PEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bethesda_pex".as_ptr(),
    mime_type: c"application/x-bethesda-pex".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Bethesda Creation Engine Compiled Papyrus Script Bytecode (.pex, 2011) Bethesda Game Studios".as_ptr(),
    extensions: [c"pex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BETHESDA_VID: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bethesda_vid".as_ptr(),
    mime_type: c"video/x-bethesda-vid".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Bethesda Softworks XnGine VID Video (.vid)".as_ptr(),
    extensions: [c"vid".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BEYONDTRUST_BRDF_RDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"beyondtrust_brdf_rdf".as_ptr(),
    mime_type: c"application/x-beyondtrust-brdf".as_ptr(),
    group: c"database".as_ptr(),
    description: c"BeyondTrust / Bomgar Remote Support Binary Resource Definition (BRDF, .rdf)".as_ptr(),
    extensions: [c"rdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BEYONDTRUST_SCC_UI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"beyondtrust_scc_ui".as_ptr(),
    mime_type: c"application/x-beyondtrust-scc-ui".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BeyondTrust Remote Support UI RPC Cache (.cache-scc-ui)".as_ptr(),
    extensions: [c"cache-scc-ui".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BFI_VIDEO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bfi_video".as_ptr(),
    mime_type: c"video/x-bfi".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Blue Byte Brute Force & Ignorance Video (.bfi, BF&I)".as_ptr(),
    extensions: [c"bfi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BFRES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bfres".as_ptr(),
    mime_type: c"application/x-nintendo-bfres".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Wii U / Switch NW4F Binary Cafe Resource Package (.bfres)".as_ptr(),
    extensions: [c"bfres".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BGI_CHR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bgi_chr".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Borland Graphics Interface Stroked Vector Font (.chr)".as_ptr(),
    extensions: [c"chr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bh".as_ptr(),
    mime_type: c"application/x-blackhole-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ZipZag BlackHole Compressed Archive (.bh)".as_ptr(),
    extensions: [c"bh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bib".as_ptr(),
    mime_type: c"text/x-bibtex".as_ptr(),
    group: c"text".as_ptr(),
    description: c"BibTeX".as_ptr(),
    extensions: [c"bib".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static BIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bif".as_ptr(),
    mime_type: c"application/x-bioware-biff".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"BioWare Infinity / Aurora Engine Binary Inclusion File Archive (.bif / .bifv, 1998) BioWare".as_ptr(),
    extensions: [c"bif".as_ptr(), c"bifv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIGWORLD_PACKED_XML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bigworld_packed_xml".as_ptr(),
    mime_type: c"application/x-bigworld-packed-xml".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BigWorld Engine Packed Binary XML (.xml / .visual_processed, 2005) BigWorld / Wargaming".as_ptr(),
    extensions: [c"xml".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIM_METADATA_FILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bim_metadata_file".as_ptr(),
    mime_type: c"application/x-bim-metadata-file".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Data Design System DDS-CAD Building Information Modeling (BIM) Metadata (.bim, PRONOM fmt/1722)".as_ptr(),
    extensions: [c"bim".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BINEXPORT2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"binexport2".as_ptr(),
    mime_type: c"application/x-binexport2".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google BinExport v2 Binary Disassembly Protobuf (.binexport, 2011) zynamics / Google".as_ptr(),
    extensions: [c"binexport".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BINK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bink".as_ptr(),
    mime_type: c"video/vnd.radgametools.bink".as_ptr(),
    group: c"video".as_ptr(),
    description: c"RAD Game Tools Bink Video Container (.bik / .bk2)".as_ptr(),
    extensions: [c"bik".as_ptr(), c"bk2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOSEMI_24_BIT_BDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"biosemi_24_bit_bdf".as_ptr(),
    mime_type: c"application/x-biosemi-bdf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"BioSemi 24-Bit BDF Electrophysiology (.bdf, PRONOM fmt/2102)".as_ptr(),
    extensions: [c"bdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOWARE_FXM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bioware_fxm".as_ptr(),
    mime_type: c"application/x-bioware-fxm".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"BioWare Eclipse / OC3 FaceFX Facial Animation (.fxm, 2009) BioWare Edmonton".as_ptr(),
    extensions: [c"fxm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOWARE_GFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bioware_gff".as_ptr(),
    mime_type: c"application/x-bioware-gff".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BioWare Aurora / Odyssey / Eclipse Generic File Format (.gff / .uti / .utc / .dlg / .are, 2002) BioWare".as_ptr(),
    extensions: [c"gff".as_ptr(), c"uti".as_ptr(), c"utc".as_ptr(), c"dlg".as_ptr(), c"are".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOWARE_KEY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bioware_key".as_ptr(),
    mime_type: c"application/x-bioware-key".as_ptr(),
    group: c"database".as_ptr(),
    description: c"BioWare Infinity / Aurora Engine Key Index (.key, 1998) BioWare".as_ptr(),
    extensions: [c"key".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOWARE_NCS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bioware_ncs".as_ptr(),
    mime_type: c"application/x-bioware-ncs".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"BioWare Aurora / Odyssey NWScript Compiled Bytecode (.ncs, 2002) BioWare".as_ptr(),
    extensions: [c"ncs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOWARE_RIM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bioware_rim".as_ptr(),
    mime_type: c"application/x-bioware-rim".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"BioWare Odyssey / Eclipse Engine Resource Image Archive (.rim / .erf)".as_ptr(),
    extensions: [c"rim".as_ptr(), c"erf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIOWARE_TLK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bioware_tlk".as_ptr(),
    mime_type: c"application/x-bioware-tlk".as_ptr(),
    group: c"database".as_ptr(),
    description: c"BioWare Aurora / Odyssey / Eclipse Talk Table (.tlk, 2002) BioWare".as_ptr(),
    extensions: [c"tlk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BITDEFENDER_AVXS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bitdefender_avxs".as_ptr(),
    mime_type: c"application/x-bitdefender-avxs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Bitdefender Antivirus Engine Threat Signature Segment (AVXS, .008 / .ivd)".as_ptr(),
    extensions: [c"ivd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BITTORRENT_BTDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bittorrent_btdb".as_ptr(),
    mime_type: c"application/x-bittorrent-btdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"BitTorrent Speed / uTorrent Ledger Database (BTComm format 0, .btdb)".as_ptr(),
    extensions: [c"btdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BITWIG_PRESET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bitwig_preset".as_ptr(),
    mime_type: c"application/x-bitwig-preset".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bitwig Studio Preset / Device / Modulator (.bwpreset / .bwdevice / .bwmodulator)".as_ptr(),
    extensions: [c"bwpreset".as_ptr(), c"bwdevice".as_ptr(), c"bwmodulator".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BIZTREE_BTD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"biztree_btd".as_ptr(),
    mime_type: c"application/x-biztree-btd".as_ptr(),
    group: c"document".as_ptr(),
    description: c"BizTree Business-in-a-Box Encrypted Document Template (.btd)".as_ptr(),
    extensions: [c"btd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLACKDRAGON_ZL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blackdragon_zl".as_ptr(),
    mime_type: c"application/x-blackdragon-zl".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"BlackDragon v3 Archive (.zl)".as_ptr(),
    extensions: [c"zl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLEND: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blend".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Blender scene".as_ptr(),
    extensions: [c"blend".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLITZ3D_FILE_FORMAT_B3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blitz3d_file_format_b3d".as_ptr(),
    mime_type: c"application/x-blitz3d-file-format-b3d".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Blitz3D File Format (.b3d, PRONOM fmt/1182)".as_ptr(),
    extensions: [c"b3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLIZZARD_M3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blizzard_m3".as_ptr(),
    mime_type: c"model/x-blizzard-m3".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Blizzard Entertainment StarCraft II M3 3D Model (.m3)".as_ptr(),
    extensions: [c"m3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLIZZARD_WOO_TBL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blizzard_woo_tbl".as_ptr(),
    mime_type: c"application/x-blizzard-woo-tbl".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Blizzard Entertainment Diablo II String Table (.tbl, 2000) Blizzard Entertainment".as_ptr(),
    extensions: [c"tbl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLIZZARD_WOW_BLS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blizzard_wow_bls".as_ptr(),
    mime_type: c"application/x-blizzard-wow-bls".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Blizzard World of Warcraft Compiled Shader Bundle (.bls)".as_ptr(),
    extensions: [c"bls".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLOSC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"blosc".as_ptr(),
    mime_type: c"application/x-blosc2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"C-Blosc2 Contiguous / Sparse Compressed Frame (.b2frame)".as_ptr(),
    extensions: [c"b2frame".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLUEBYTE_TPWM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bluebyte_tpwm".as_ptr(),
    mime_type: c"application/x-bluebyte-tpwm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Blue Byte Turbo Packer Wolfgang Mayerhofer (.tpwm)".as_ptr(),
    extensions: [c"tpwm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLUESTRIPE_HB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bluestripe_hb".as_ptr(),
    mime_type: c"application/x-bluestripe-hb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BlueStripe FactFinder / Microsoft Monitoring BlackBox Capture (.hb) BlueStripe Software".as_ptr(),
    extensions: [c"hb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLURAY_CLPI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bluray_clpi".as_ptr(),
    mime_type: c"application/x-bluray-clpi".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Blu-ray Disc Clip Information (.clpi)".as_ptr(),
    extensions: [c"clpi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLURAY_MOBJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bluray_mobj".as_ptr(),
    mime_type: c"application/x-bluray-mobj".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Blu-ray Disc MovieObject Navigation Commands (MovieObject.bdmv)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BLURAY_MPLS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bluray_mpls".as_ptr(),
    mime_type: c"application/x-bluray-mpls".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Blu-ray Disc Movie PlayList (.mpls)".as_ptr(),
    extensions: [c"mpls".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BMF_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bmf_font".as_ptr(),
    mime_type: c"font/x-bmf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Tomaž Kac / CRS ByteMap Font (.bmf, 1998) Tomaž Kac".as_ptr(),
    extensions: [c"bmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bmp".as_ptr(),
    mime_type: c"image/bmp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"BMP image data".as_ptr(),
    extensions: [c"bmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BMX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bmx".as_ptr(),
    mime_type: c"audio/x-bmx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Oskari Tammelin Jeskola Buzz Modular Song (.bmx / .bmw)".as_ptr(),
    extensions: [c"bmx".as_ptr(), c"bmw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bng".as_ptr(),
    mime_type: c"image/x-bng".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Better than PNG Lossless Image (.bng)".as_ptr(),
    extensions: [c"bng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BNS_DIALOG_CBF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bns_dialog_cbf".as_ptr(),
    mime_type: c"application/x-bns-dialog-cbf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Blade & Soul Dialog Screenshots Bundle (.cbf)".as_ptr(),
    extensions: [c"cbf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BNTL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bntl".as_ptr(),
    mime_type: c"application/x-binja-bntl".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Vector 35 Binary Ninja Type Library (.bntl)".as_ptr(),
    extensions: [c"bntl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOCHS_DISK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bochs_disk".as_ptr(),
    mime_type: c"application/x-bochs-disk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Bochs x86 Emulator Growing Virtual HD Image (Bochs Virtual HD Image)".as_ptr(),
    extensions: [c"bochs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BODYSLIDE_OSD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bodyslide_osd".as_ptr(),
    mime_type: c"model/x-bodyslide-osd".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"BodySlide Outfit Studio Morph Diff Data (.osd)".as_ptr(),
    extensions: [c"osd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOHEMIA_P3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bohemia_p3d".as_ptr(),
    mime_type: c"model/x-bohemia-p3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Bohemia Interactive Real Virtuality P3D 3D Model (.p3d, 2001) Bohemia Interactive".as_ptr(),
    extensions: [c"p3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOHEMIA_PAA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bohemia_paa".as_ptr(),
    mime_type: c"image/x-bohemia-paa".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Bohemia Interactive Real Virtuality PAA / PAC Texture (.paa / .pac, 2001) Bohemia Interactive".as_ptr(),
    extensions: [c"paa".as_ptr(), c"pac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOHEMIA_RTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bohemia_rtm".as_ptr(),
    mime_type: c"application/x-bohemia-rtm".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Bohemia Interactive Real Virtuality Skeletal Animation (.rtm, 2001) Bohemia Interactive".as_ptr(),
    extensions: [c"rtm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOLT_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bolt_db".as_ptr(),
    mime_type: c"application/x-bolt-db".as_ptr(),
    group: c"database".as_ptr(),
    description: c"BoltDB / bbolt Go Key-Value Database (.bolt / .bdb)".as_ptr(),
    extensions: [c"bolt".as_ptr(), c"bdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bom".as_ptr(),
    mime_type: c"application/x-apple-bom".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple / NeXTSTEP Bill of Materials (BOMStore)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOOMTRACKER_CFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"boomtracker_cff".as_ptr(),
    mime_type: c"audio/x-boomtracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"CUD BoomTracker 4.0 AdLib OPL2 Module (.cff)".as_ptr(),
    extensions: [c"cff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BORLAND_BDE_LOCALE_BTL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"borland_bde_locale_btl".as_ptr(),
    mime_type: c"application/x-borland-bde-locale-btl".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Borland Database Engine (BDE) Locale Driver (.btl, 1994) Borland".as_ptr(),
    extensions: [c"btl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BORLAND_BGI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"borland_bgi".as_ptr(),
    mime_type: c"application/x-borland-bgi".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland Graphics Interface Video Driver (.bgi, 1987) Borland".as_ptr(),
    extensions: [c"bgi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BORLAND_DPMI_OVERLAY_OVY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"borland_dpmi_overlay_ovy".as_ptr(),
    mime_type: c"application/x-borland-dpmi-overlay-ovy".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland C++ / Pascal DPMI Protected-Mode Overlay (.ovy, 1991) Borland".as_ptr(),
    extensions: [c"ovy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BORLAND_TPU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"borland_tpu".as_ptr(),
    mime_type: c"application/x-borland-tpu".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland Turbo Pascal Compiled Unit (.tpu / .tpp / .tpw, 1987) Borland".as_ptr(),
    extensions: [c"tpu".as_ptr(), c"tpp".as_ptr(), c"tpw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BORLAND_TURBO_DEBUG_HELP_TDH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"borland_turbo_debug_help_tdh".as_ptr(),
    mime_type: c"application/x-borland-turbo-debug-help-tdh".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Borland Turbo Debugger Help File (.tdh, 1988) Borland".as_ptr(),
    extensions: [c"tdh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BORLAND_TURBO_HELP_TCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"borland_turbo_help_tch".as_ptr(),
    mime_type: c"application/x-borland-turbo-help-tch".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Borland Turbo C / Turbo Assembler Help File (.tch / .tah, 1988) Borland".as_ptr(),
    extensions: [c"tch".as_ptr(), c"tah".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOX86_86F: MagikaTypeInfo = MagikaTypeInfo {
    label: c"box86_86f".as_ptr(),
    mime_type: c"application/x-86box-86f".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"86Box Emulator Surface Floppy Disk Image (.86f, 86BF)".as_ptr(),
    extensions: [c"86f".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BOYSCOUT_BSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"boyscout_bsf".as_ptr(),
    mime_type: c"audio/x-boyscout-bsf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"PID BoyScout Game Boy Chiptune Module (.bsf)".as_ptr(),
    extensions: [c"bsf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BPG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bpg".as_ptr(),
    mime_type: c"image/bpg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"BPG".as_ptr(),
    extensions: [c"bpg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BPL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bpl".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"bpl".as_ptr(),
    extensions: [c"bpl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BR_AS3TRACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"br_as3trace".as_ptr(),
    mime_type: c"application/x-br-as3trace".as_ptr(),
    group: c"application".as_ptr(),
    description: c"B&R Industrial Automation Studio 3 Trace Capture (.as3trace) B&R Industrial Automation".as_ptr(),
    extensions: [c"as3trace".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BROADCOM_CFE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"broadcom_cfe".as_ptr(),
    mime_type: c"application/x-broadcom-cfe".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Broadcom CFE ImageTag Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BROADCOM_HCD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"broadcom_hcd".as_ptr(),
    mime_type: c"application/x-broadcom-hcd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Broadcom / Cypress Bluetooth HCI RAM Patch Download Image (.hcd)".as_ptr(),
    extensions: [c"hcd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BROADCOM_STB_BOOTLOADER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"broadcom_stb_bootloader".as_ptr(),
    mime_type: c"application/x-broadcom-bootloader".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Broadcom Set-Top Box Bootloader Image (%BL!)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BROADCOM_TRX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"broadcom_trx".as_ptr(),
    mime_type: c"application/x-broadcom-trx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Broadcom TRX Router Firmware Image (HDR0)".as_ptr(),
    extensions: [c"trx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BROTHER_PES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"brother_pes".as_ptr(),
    mime_type: c"application/x-brother-pes".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Brother / Babylock / Bernina PES Embroidery Design (.pes)".as_ptr(),
    extensions: [c"pes".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BRSAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"brsar".as_ptr(),
    mime_type: c"application/x-nintendo-rsar".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo GameCube / Wii Sound Archive (.brsar)".as_ptr(),
    extensions: [c"brsar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BRSTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"brstm".as_ptr(),
    mime_type: c"audio/x-brstm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo Wii Binary Revolution Stream Audio (.brstm)".as_ptr(),
    extensions: [c"brstm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BRUNSLI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"brunsli".as_ptr(),
    mime_type: c"image/x-brunsli".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Google Brunsli Lossless Repacked JPEG (.brn)".as_ptr(),
    extensions: [c"brn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BRWAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"brwav".as_ptr(),
    mime_type: c"audio/x-brwav".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo Wii Binary Revolution Wave Audio (.brwav)".as_ptr(),
    extensions: [c"brwav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BSA_BA2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bsa_ba2".as_ptr(),
    mime_type: c"application/x-bethesda-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Bethesda Game Studios Creation Engine Archive (.bsa / .ba2)".as_ptr(),
    extensions: [c"bsa".as_ptr(), c"ba2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BSD_DUMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bsd_dump".as_ptr(),
    mime_type: c"application/x-bsd-dump".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"BSD / Unix UFS dump(8) Backup Archive (.dump, 1983) UC Berkeley CSRG".as_ptr(),
    extensions: [c"dump".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BSDIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bsdiff".as_ptr(),
    mime_type: c"application/x-bsdiff".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Colin Percival bsdiff / Chromium Courgette Binary Patch".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BSON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bson".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BSON".as_ptr(),
    extensions: [c"bson".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BSP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bsp".as_ptr(),
    mime_type: c"model/x-bsp".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Valve Source / Quake II / III Binary Space Partitioning Map (.bsp)".as_ptr(),
    extensions: [c"bsp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BTRFS_SEND_STREAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"btrfs_send_stream".as_ptr(),
    mime_type: c"application/x-btrfs-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Linux Btrfs Incremental Subvolume Send Stream (btrfs-stream\\0)".as_ptr(),
    extensions: [c"stream".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BUN_PILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bun_pile".as_ptr(),
    mime_type: c"application/x-bun-pile".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Bun JavaScript Runtime Transpiler Cache (.pile)".as_ptr(),
    extensions: [c"pile".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BURNPIPE_BA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"burnpipe_ba".as_ptr(),
    mime_type: c"application/x-burnpipe-ba".as_ptr(),
    group: c"application".as_ptr(),
    description: c"WiX Burn Elevation Pipe Manifest (.ba)".as_ptr(),
    extensions: [c"ba".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BUZZIC2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"buzzic2".as_ptr(),
    mime_type: c"audio/x-buz2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Rebels Buzzic 2 Synthesizer Module (.buz2)".as_ptr(),
    extensions: [c"buz2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BWD1_DOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bwd1_doc".as_ptr(),
    mime_type: c"application/x-bwd1-doc".as_ptr(),
    group: c"document".as_ptr(),
    description: c"BWD1 Structured Document (.bwd)".as_ptr(),
    extensions: [c"bwd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bzip".as_ptr(),
    mime_type: c"application/x-bzip2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"bzip2 compressed data".as_ptr(),
    extensions: [c"bz2".as_ptr(), c"tbz2".as_ptr(), c"tar.bz2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static BZIP3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"bzip3".as_ptr(),
    mime_type: c"application/x-bzip3".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"bzip3 compressed data".as_ptr(),
    extensions: [c"bz3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static C: MagikaTypeInfo = MagikaTypeInfo {
    label: c"c".as_ptr(),
    mime_type: c"text/x-c".as_ptr(),
    group: c"code".as_ptr(),
    description: c"C source".as_ptr(),
    extensions: [c"c".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static C1581_D81: MagikaTypeInfo = MagikaTypeInfo {
    label: c"c1581_d81".as_ptr(),
    mime_type: c"application/x-c64-d81".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Commodore 1581 3.5-Inch 800 KB Floppy Disk Image (.d81)".as_ptr(),
    extensions: [c"d81".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static C3_MAXFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"c3_maxfile".as_ptr(),
    mime_type: c"application/x-c3-maxfile".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"TQ Digital C3 Engine 3D Physics Mesh (.c3, 2003) NetDragon / TQ Digital".as_ptr(),
    extensions: [c"c3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static C64_GODOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"c64_godot".as_ptr(),
    mime_type: c"image/x-c64-godot".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Commodore 64 GoDot Image / Clip (.4bt / .clp, GOD0/GOD1)".as_ptr(),
    extensions: [c"4bt".as_ptr(), c"clp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static C64_TAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"c64_tap".as_ptr(),
    mime_type: c"application/x-c64-tap".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Commodore 64 / C16 CCS64 Raw Datasette Tape Pulse Image (.tap)".as_ptr(),
    extensions: [c"tap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cab".as_ptr(),
    mime_type: c"application/vnd.ms-cab-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Cabinet archive data".as_ptr(),
    extensions: [c"cab".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CADCORP_SIS_NOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cadcorp_sis_nol".as_ptr(),
    mime_type: c"application/x-cadcorp-sis-nol".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Cadcorp SIS Named Object Library (.nol)".as_ptr(),
    extensions: [c"nol".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"caf".as_ptr(),
    mime_type: c"audio/x-caf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Apple Core Audio Format (.caf, PRONOM fmt/416, LoC fdd000591)".as_ptr(),
    extensions: [c"caf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAFFEINE_CSCHEME: MagikaTypeInfo = MagikaTypeInfo {
    label: c"caffeine_cscheme".as_ptr(),
    mime_type: c"application/x-caffeine-cscheme".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Caffeine.Scheme Database Schema (.cscheme)".as_ptr(),
    extensions: [c"cscheme".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAKEWALK_WRK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cakewalk_wrk".as_ptr(),
    mime_type: c"audio/x-cakewalk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Twelve Tone Systems Cakewalk Project (.wrk)".as_ptr(),
    extensions: [c"wrk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAL3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cal3d".as_ptr(),
    mime_type: c"model/x-cal3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Cal3D 3D Character Skeleton / Mesh / Animation / Material (.csf / .cmf / .caf / .crf)".as_ptr(),
    extensions: [c"csf".as_ptr(), c"cmf".as_ptr(), c"caf".as_ptr(), c"crf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CALS_DCL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cals_dcl".as_ptr(),
    mime_type: c"application/x-cals-dcl".as_ptr(),
    group: c"document".as_ptr(),
    description: c"CALS MIL-STD-1840 Raster Declaration File (.dcl)".as_ptr(),
    extensions: [c"dcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CANON_VRD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"canon_vrd".as_ptr(),
    mime_type: c"application/x-canon-vrd".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Canon Digital Photo Professional Recipe Data (.vrd / .dr4)".as_ptr(),
    extensions: [c"vrd".as_ptr(), c"dr4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAPCOM_MT_GMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"capcom_mt_gmd".as_ptr(),
    mime_type: c"application/x-capcom-mt-gmd".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Capcom MT Framework Localized Text Message Database (.gmd, 2009) Capcom".as_ptr(),
    extensions: [c"gmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAPCOM_MT_MRL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"capcom_mt_mrl".as_ptr(),
    mime_type: c"model/x-capcom-mt-mrl".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Capcom MT Framework Material Library (.mrl, 2006) Capcom".as_ptr(),
    extensions: [c"mrl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAPCOM_RE_PAK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"capcom_re_pak".as_ptr(),
    mime_type: c"application/x-capcom-re-pak".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Capcom RE Engine KPKA Game Archive (.pak, 2017) Capcom".as_ptr(),
    extensions: [c"pak".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAPS_IPF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"caps_ipf".as_ptr(),
    mime_type: c"application/x-caps-ipf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Software Preservation Society (SPS / CAPS) Interchangeable Preservation Format Flux Image (.ipf)".as_ptr(),
    extensions: [c"ipf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CARV_VRAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"carv_vrac".as_ptr(),
    mime_type: c"application/x-carv-vrac".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CARV VRAC Metadata Container (.vrac)".as_ptr(),
    extensions: [c"vrac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CASIO_CONTAINER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"casio_container".as_ptr(),
    mime_type: c"application/x-casio-container".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Casio Graphing Calculator Bitwise-Inverted USBPower Container (.g1a / .g1m / .g3a / .g1e / .c2m)".as_ptr(),
    extensions: [c"g1a".as_ptr(), c"g1m".as_ptr(), c"g3a".as_ptr(), c"g1e".as_ptr(), c"c2m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CASLOG_DLOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"caslog_dlog".as_ptr(),
    mime_type: c"application/x-caslog-v1".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CASLOGv1 Binary Diagnostic / Usage Log (.dlog / .ulog)".as_ptr(),
    extensions: [c"dlog".as_ptr(), c"ulog".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CASTLEDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"castledb".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CastleDB".as_ptr(),
    extensions: [c"cdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cat".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Catalog file".as_ptr(),
    extensions: [c"cat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CATBOOST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"catboost".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CatBoost model".as_ptr(),
    extensions: [c"cbm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CATIA_DRAWING: MagikaTypeInfo = MagikaTypeInfo {
    label: c"catia_drawing".as_ptr(),
    mime_type: c"application/x-catia-drawing".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Dassault Systèmes CATIA V5 Drawing (.catdrawing, PRONOM fmt/1615)".as_ptr(),
    extensions: [c"catdrawing".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAUSTIC_PCMSYNTH_WK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"caustic_pcmsynth_wk".as_ptr(),
    mime_type: c"audio/x-caustic-pcmsynth".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Single Cell Software Caustic PCMSynth Preset (.pcmsynth)".as_ptr(),
    extensions: [c"pcmsynth".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CAZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cazip".as_ptr(),
    mime_type: c"application/x-cazip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Computer Associates CAZIP Compressed File".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CBA_MODULE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cba_module".as_ptr(),
    mime_type: c"audio/x-cba".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Chuck Biscuits / Black Artist Tracker Module (.cba)".as_ptr(),
    extensions: [c"cba".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CBOR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cbor".as_ptr(),
    mime_type: c"application/cbor".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CBOR".as_ptr(),
    extensions: [c"cbor".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CCZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ccz".as_ptr(),
    mime_type: c"application/x-cocos2d-ccz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Cocos2d / TexturePacker Compressed PVR Container (.ccz)".as_ptr(),
    extensions: [c"ccz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CDES_ENCRYPTED_ENC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cdes_encrypted_enc".as_ptr(),
    mime_type: c"application/x-cdes-encrypted-enc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CDES Encrypted File Container (.enc)".as_ptr(),
    extensions: [c"enc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cdf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"cdf".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CDFM_670: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cdfm_670".as_ptr(),
    mime_type: c"audio/x-cdfm-670".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Renaissance CDFM / Composer 670 Packed Module (.670)".as_ptr(),
    extensions: [c"670".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CDTREE_CD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cdtree_cd".as_ptr(),
    mime_type: c"application/x-cdtree-cd".as_ptr(),
    group: c"database".as_ptr(),
    description: c"cdTree Offline Media Catalog Database (.cd)".as_ptr(),
    extensions: [c"cd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cdx".as_ptr(),
    mime_type: c"chemical/x-cdx".as_ptr(),
    group: c"document".as_ptr(),
    description: c"ChemDraw Chemical Draw Exchange (.cdx, PRONOM fmt/378, LoC fdd000582)".as_ptr(),
    extensions: [c"cdx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CEDICT_UNIT_U8: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cedict_unit_u8".as_ptr(),
    mime_type: c"application/x-cedict-unit-u8".as_ptr(),
    group: c"database".as_ptr(),
    description: c"CC-CEDICT ASCII-0x1F Unit-Delimited Chinese Dictionary (.u8)".as_ptr(),
    extensions: [c"u8".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CEDIT_UCVIEW_HIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cedit_ucview_his".as_ptr(),
    mime_type: c"application/x-cedit-ucview-his".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Chinese DOS CEdit / UCVIEW Revision History (.his)".as_ptr(),
    extensions: [c"his".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CEDIT_UCVIEW_HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cedit_ucview_hlp".as_ptr(),
    mime_type: c"application/x-cedit-ucview-hlp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Chinese DOS CEdit / UCVIEW Help File (.hlp)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CERN_ROOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cern_root".as_ptr(),
    mime_type: c"application/x-cern-root".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CERN ROOT Binary Data File (.root)".as_ptr(),
    extensions: [c"root".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CERN_ZEBRA_FZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cern_zebra_fz".as_ptr(),
    mime_type: c"application/x-cern-zebra-fz".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CERNLIB ZEBRA FZ Sequential Exchange File (.fz / .xsdst)".as_ptr(),
    extensions: [c"fz".as_ptr(), c"xsdst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CERN_ZEBRA_RZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cern_zebra_rz".as_ptr(),
    mime_type: c"application/x-cern-zebra-rz".as_ptr(),
    group: c"database".as_ptr(),
    description: c"CERNLIB ZEBRA RZ Direct-Access File / HBOOK (.rz / .hbook / .dat)".as_ptr(),
    extensions: [c"rz".as_ptr(), c"hbook".as_ptr(), c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CESIUM_B3DM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cesium_b3dm".as_ptr(),
    mime_type: c"model/gltf-binary".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Cesium OGC 3D Tiles Batched 3D Model (.b3dm)".as_ptr(),
    extensions: [c"b3dm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CESIUM_GEOM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cesium_geom".as_ptr(),
    mime_type: c"application/x-cesium-geom".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Cesium 3D Tiles Geometry Tile (.geom)".as_ptr(),
    extensions: [c"geom".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CFL3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cfl3".as_ptr(),
    mime_type: c"application/x-cfl3-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Jari Komppa Compressed File Library v3 (.cfl)".as_ptr(),
    extensions: [c"cfl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHAPS_TIMETABLE_TT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chaps_timetable_tt".as_ptr(),
    mime_type: c"application/x-chaps-timetable-tt".as_ptr(),
    group: c"database".as_ptr(),
    description: c"CHAPS TimeTable Version 004 Transit Schedule Database (.tt)".as_ptr(),
    extensions: [c"tt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chd".as_ptr(),
    mime_type: c"application/x-chd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MAME / MESS Compressed Hunks of Data (CHD)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHEMSTATION_GAS_CHROMATOGRAPHY_MASS_MS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chemstation_gas_chromatography_mass_ms".as_ptr(),
    mime_type: c"application/x-chemstation-gas-chromatography-mass-ms".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Agilent / HP ChemStation Gas Chromatography / Mass Spectrometry (GC/MS) Data (.ms, PRONOM fmt/2101)".as_ptr(),
    extensions: [c"ms".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHESSBASE_TREE_CTB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chessbase_tree_ctb".as_ptr(),
    mime_type: c"application/x-chessbase-tree-ctb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"ChessBase Opening Book Tree Table (.ctb)".as_ptr(),
    extensions: [c"ctb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHIPTRACKER_KRIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chiptracker_kris".as_ptr(),
    mime_type: c"audio/x-chiptracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Kris Hatlelid Chiptracker Amiga Module (.mod / .kris)".as_ptr(),
    extensions: [c"mod".as_ptr(), c"kris".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chm".as_ptr(),
    mime_type: c"application/chm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows HtmlHelp Data".as_ptr(),
    extensions: [c"chm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROME_BDIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chrome_bdic".as_ptr(),
    mime_type: c"application/x-chrome-bdic".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Google Chrome Compiled Hunspell Spellcheck Dictionary (.bdic)".as_ptr(),
    extensions: [c"bdic".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROME_CACHE_BLOCKFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chrome_cache_blockfile".as_ptr(),
    mime_type: c"application/x-chromium-cache-blockfile".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Google Chromium Disk Cache v2 Blockfile (data_0..data_3)".as_ptr(),
    extensions: [c"data_3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROME_SIMPLE_CACHE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chrome_simple_cache".as_ptr(),
    mime_type: c"application/x-chromium-simple-cache".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Google Chromium Simple Cache Stream Entry (_0.._9)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROME_SNSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chrome_snss".as_ptr(),
    mime_type: c"application/x-chrome-snss".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google Chrome / Chromium Session & Tab Restore File (SNSS)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROME_V4STORE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chrome_v4store".as_ptr(),
    mime_type: c"application/x-chrome-safebrowsing-store".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Google Chrome Safe Browsing V4 Store (.store, 0x600D71FE)".as_ptr(),
    extensions: [c"store".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROME_VLNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chrome_vlnk".as_ptr(),
    mime_type: c"application/x-chrome-visited-links".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Google Chrome / Chromium Visited Links History (VLnk)".as_ptr(),
    extensions: [c"vlnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROMEOS_FMAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chromeos_fmap".as_ptr(),
    mime_type: c"application/x-chromeos-fmap".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google ChromeOS / Coreboot Flash Map (__FMAP__)".as_ptr(),
    extensions: [c"fmap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROMEOS_VB21: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chromeos_vb21".as_ptr(),
    mime_type: c"application/x-chromeos-vb21".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google ChromeOS Verified Boot 2.1 Structure (Vb2B / Vb2F / Vb2I / Vb2K / Vb2P / Vb2S, 2014) ChromiumOS".as_ptr(),
    extensions: [c"vb2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHROMEOS_VBOOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chromeos_vboot".as_ptr(),
    mime_type: c"application/x-chromeos-vboot".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google ChromeOS Verified Boot Keyblock (CHROMEOS)".as_ptr(),
    extensions: [c"keyblock".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHRONOMASTER_DFA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chronomaster_dfa".as_ptr(),
    mime_type: c"video/x-dfa".as_ptr(),
    group: c"video".as_ptr(),
    description: c"DreamForge Chronomaster DFIA Animation Video (.dfa)".as_ptr(),
    extensions: [c"dfa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CHX_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"chx_font".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"CHX Big Font (.chx)".as_ptr(),
    extensions: [c"chx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CIGAMES_DPK4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cigames_dpk4".as_ptr(),
    mime_type: c"application/x-cigames-dpk4".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CI Games DPK4 Package Archive (.dpk)".as_ptr(),
    extensions: [c"dpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CINEMA4D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cinema4d".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Cinema 4D scene".as_ptr(),
    extensions: [c"c4d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CINEON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cineon".as_ptr(),
    mime_type: c"image/x-cineon".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Kodak Cineon Digital Film Image (.cin)".as_ptr(),
    extensions: [c"cin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CIV3_BIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"civ3_bic".as_ptr(),
    mime_type: c"application/x-civ3-bic".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Firaxis Games Civilization III Editor Scenario (.bic / .bix / .biq, 2001) Firaxis Games".as_ptr(),
    extensions: [c"bic".as_ptr(), c"bix".as_ptr(), c"biq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLANG_OFFLOAD_BUNDLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clang_offload_bundle".as_ptr(),
    mime_type: c"application/x-clang-offload-bundle".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Clang Offload Bundle (CCOB / __CLANG_OFFLOAD_BUNDLE__)".as_ptr(),
    extensions: [c"bundle".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLARION_HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clarion_hlp".as_ptr(),
    mime_type: c"application/x-clarion-hlp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Barrington Systems Clarion for DOS Help File (.hlp)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLARION_MEM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clarion_mem".as_ptr(),
    mime_type: c"application/x-clarion-mem".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Barrington Systems Clarion for DOS Memo File (.mem)".as_ptr(),
    extensions: [c"mem".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLARIS_RESOLVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"claris_resolve".as_ptr(),
    mime_type: c"application/x-claris-resolve".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Claris Resolve Macintosh Spreadsheet (.rsv, BAT_BTSS)".as_ptr(),
    extensions: [c"rsv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLICKTEAM_CCN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clickteam_ccn".as_ptr(),
    mime_type: c"application/x-clickteam-ccn".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Clickteam Multimedia Fusion / Fusion 2.5 Sub-Application (.ccn / .cca, 2006) Clickteam".as_ptr(),
    extensions: [c"ccn".as_ptr(), c"cca".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLICKTEAM_MFA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clickteam_mfa".as_ptr(),
    mime_type: c"application/x-clickteam-mfa".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Clickteam Fusion / Multimedia Fusion 2 Unicode Project (.mfa, 2006) Clickteam".as_ptr(),
    extensions: [c"mfa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLIP_STUDIO_PAINT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clip_studio_paint".as_ptr(),
    mime_type: c"application/x-clip-studio-paint".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Celsys Clip Studio Paint Illustration Document (.clip)".as_ptr(),
    extensions: [c"clip".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLIPPER_NTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clipper_ntx".as_ptr(),
    mime_type: c"application/x-clipper-ntx".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Nantucket / CA-Clipper NTX Database Index (.ntx, 1987) Nantucket / Computer Associates".as_ptr(),
    extensions: [c"ntx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CLOJURE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"clojure".as_ptr(),
    mime_type: c"text/x-clojure".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Clojure".as_ptr(),
    extensions: [c"clj".as_ptr(), c"cljs".as_ptr(), c"cljc".as_ptr(), c"cljr".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CMAKE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cmake".as_ptr(),
    mime_type: c"text/x-cmake".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CMake build file".as_ptr(),
    extensions: [c"cmake".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CMAP_CMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cmap_cmp".as_ptr(),
    mime_type: c"application/x-cmap-cmp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CMAP Zlib-Compressed Collision Tile Map (.cmp)".as_ptr(),
    extensions: [c"cmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CMX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cmx".as_ptr(),
    mime_type: c"image/x-cmx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Corel Presentation Exchange Metafile (CMX)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COBOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cobol".as_ptr(),
    mime_type: c"text/x-cobol".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Cobol".as_ptr(),
    extensions: [c"cbl".as_ptr(), c"cob".as_ptr(), c"cpy".as_ptr(), c"CBL".as_ptr(), c"COB".as_ptr(), c"CPY".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static COD_FASTFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cod_fastfile".as_ptr(),
    mime_type: c"application/x-cod-fastfile".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Infinity Ward / Activision Call of Duty FastFile Archive (.ff / .fp, 2007) Infinity Ward".as_ptr(),
    extensions: [c"ff".as_ptr(), c"fp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COD_GSCC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cod_gscc".as_ptr(),
    mime_type: c"application/x-cod-gscc".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Activision Call of Duty Compiled GSC Script Bytecode (.gscc)".as_ptr(),
    extensions: [c"gscc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"coff".as_ptr(),
    mime_type: c"application/x-coff".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Intel 80386 COFF".as_ptr(),
    extensions: [c"obj".as_ptr(), c"o".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COFFEESCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"coffeescript".as_ptr(),
    mime_type: c"text/coffeescript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CoffeeScript".as_ptr(),
    extensions: [c"coffee".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static COKTEL_STK2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"coktel_stk2".as_ptr(),
    mime_type: c"application/x-coktel-stk2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Coktel Vision STK21 Archive (.stk / .itk)".as_ptr(),
    extensions: [c"stk".as_ptr(), c"itk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COLLADA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"collada".as_ptr(),
    mime_type: c"model/vnd.collada+xml".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"COLLADA digital asset".as_ptr(),
    extensions: [c"dae".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static COMBILOADER_BCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"combiloader_bcf".as_ptr(),
    mime_type: c"application/x-combiloader-bcf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SMS-Soft CombiLoader Binary Container File (.bcf) SMS-Soft".as_ptr(),
    extensions: [c"bcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COMMON_INSTRUMENT_CI1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"common_instrument_ci1".as_ptr(),
    mime_type: c"application/x-common-instrument-cif".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Common Instrument File Format (.ci1 / .ci2, PRONOM fmt/1887)".as_ptr(),
    extensions: [c"ci1".as_ptr(), c"ci2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COMMON_LOUDSPEAKER_CF1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"common_loudspeaker_cf1".as_ptr(),
    mime_type: c"application/x-common-loudspeaker-clf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Common Loudspeaker Format Binary (.cf1 / .cf2, PRONOM fmt/1944)".as_ptr(),
    extensions: [c"cf1".as_ptr(), c"cf2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COMPOSER_667: MagikaTypeInfo = MagikaTypeInfo {
    label: c"composer_667".as_ptr(),
    mime_type: c"audio/x-composer-667".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Future Crew Composer 667 OPL2 Module (.667)".as_ptr(),
    extensions: [c"667".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CONS_DICT_NX2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cons_dict_nx2".as_ptr(),
    mime_type: c"application/x-cons-dict-nx2".as_ptr(),
    group: c"database".as_ptr(),
    description: c"CONS+ DICT(HUGE) Dictionary Database (.nx2)".as_ptr(),
    extensions: [c"nx2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CONSULTANTPLUS_DT5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"consultantplus_dt5".as_ptr(),
    mime_type: c"application/x-consultantplus-dt5".as_ptr(),
    group: c"database".as_ptr(),
    description: c"ConsultantPlus Information Bank Database (.dt5)".as_ptr(),
    extensions: [c"dt5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COREBOOT_CBFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"coreboot_cbfs".as_ptr(),
    mime_type: c"application/x-coreboot-cbfs".as_ptr(),
    group: c"application".as_ptr(),
    description: c"coreboot CBFS ROM Component File (LARCHIVE)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COSMIGO_BRC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cosmigo_brc".as_ptr(),
    mime_type: c"image/x-cosmigo-brc".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Cosmigo Pro Motion Brush Container (.brc, 2012) Cosmigo".as_ptr(),
    extensions: [c"brc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static COSMIGO_STM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cosmigo_stm".as_ptr(),
    mime_type: c"image/x-cosmigo-stm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Cosmigo Pro Motion Simple Tile Map (.stm, 2017) Cosmigo".as_ptr(),
    extensions: [c"stm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CPIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cpio".as_ptr(),
    mime_type: c"application/x-cpio".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CPIO archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cpk".as_ptr(),
    mime_type: c"application/x-criware-cpk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CRI Middleware CRIWARE FileMajik PRO Package (.cpk)".as_ptr(),
    extensions: [c"cpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cpp".as_ptr(),
    mime_type: c"text/x-c".as_ptr(),
    group: c"code".as_ptr(),
    description: c"C++ source".as_ptr(),
    extensions: [c"cc".as_ptr(), c"cpp".as_ptr(), c"cxx".as_ptr(), c"c++".as_ptr(), c"cppm".as_ptr(), c"ixx".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cpx".as_ptr(),
    mime_type: c"application/x-coreldraw-cpx".as_ptr(),
    group: c"document".as_ptr(),
    description: c"CorelDRAW Compressed Drawing (.cpx, PRONOM x-fmt/36)".as_ptr(),
    extensions: [c"cpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cram".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"CRAM alignment".as_ptr(),
    extensions: [c"cram".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRAMFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cramfs".as_ptr(),
    mime_type: c"application/x-cramfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Linux Compressed ROM File System Image (.cramfs)".as_ptr(),
    extensions: [c"cramfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRASHPAD_CPAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crashpad_cpad".as_ptr(),
    mime_type: c"application/x-crashpad-settings".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google Chrome Crashpad Database Settings (CPAD)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CREATIVE_XFI_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"creative_xfi_fw".as_ptr(),
    mime_type: c"application/x-creative-xfi-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Creative Sound Blaster X-Fi / CA0132 DSP Microcode Container (ctefx.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRI_HCA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cri_hca".as_ptr(),
    mime_type: c"audio/x-cri-hca".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"CRI Middleware High Compression Audio (.hca)".as_ptr(),
    extensions: [c"hca".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRI_USM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cri_usm".as_ptr(),
    mime_type: c"video/x-cri-usm".as_ptr(),
    group: c"video".as_ptr(),
    description: c"CRI Middleware Sofdec2 USM Video Container (.usm)".as_ptr(),
    extensions: [c"usm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRIWARE_AWB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"criware_awb".as_ptr(),
    mime_type: c"audio/x-criware-awb".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"CRI Middleware CRIWARE AFS2 / ADX2 Wave Bank Archive (.awb, 2009) CRI Middleware".as_ptr(),
    extensions: [c"awb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crt".as_ptr(),
    mime_type: c"application/x-x509-ca-cert".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Certificates (binary format)".as_ptr(),
    extensions: [c"der".as_ptr(), c"cer".as_ptr(), c"crt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRUNCH_MANIA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crunch_mania".as_ptr(),
    mime_type: c"application/x-crunchmania".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Thomas Schwarz Crunch-Mania Amiga Data Archive (.crm)".as_ptr(),
    extensions: [c"crm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRUSH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crush".as_ptr(),
    mime_type: c"application/x-crush".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"PocketWare CRUSH Archive / Index (.cru / .cri)".as_ptr(),
    extensions: [c"cru".as_ptr(), c"cri".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crx".as_ptr(),
    mime_type: c"application/x-chrome-extension".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Google Chrome extension".as_ptr(),
    extensions: [c"crx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYENGINE_DDS_MIP6: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryengine_dds_mip6".as_ptr(),
    mime_type: c"application/x-cryengine-dds-mip6".as_ptr(),
    group: c"image".as_ptr(),
    description: c"CryEngine Split DDS Mipmap Stream (.dds.6)".as_ptr(),
    extensions: [c"dds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYENGINE_SOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryengine_soc".as_ptr(),
    mime_type: c"application/x-cryengine-soc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Crytek CryEngine CrChF Object Container (.soc / .cgf, 2004) Crytek".as_ptr(),
    extensions: [c"soc".as_ptr(), c"cgf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYO_APC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryo_apc".as_ptr(),
    mime_type: c"audio/x-cryo-apc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Cryo Interactive IMA ADPCM Audio (.apc, CRYO_APC)".as_ptr(),
    extensions: [c"apc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYO_BIGFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryo_bigfile".as_ptr(),
    mime_type: c"application/x-cryo-bigfile".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Cryo Interactive BigFile Game Archive (.big, 1998) Cryo Interactive".as_ptr(),
    extensions: [c"big".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYO_CLIB_VOX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryo_clib_vox".as_ptr(),
    mime_type: c"application/x-cryo-clib-vox".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Adventure Game Studio CLIB Resource Archive (.ags / .vox, 1999) Chris Jones".as_ptr(),
    extensions: [c"ags".as_ptr(), c"vox".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYO_HNM4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryo_hnm4".as_ptr(),
    mime_type: c"video/x-hnm4".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Cryo Interactive HNM4 Multimedia Video (.hnm)".as_ptr(),
    extensions: [c"hnm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYPTOAPI_KEYBLOB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cryptoapi_keyblob".as_ptr(),
    mime_type: c"application/x-cryptoapi-keyblob".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft CryptoAPI RSA key BLOB (PUBLICKEYBLOB / PRIVATEKEYBLOB, .snk)".as_ptr(),
    extensions: [c"snk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CRYSTAL_REPORTS_V4_RPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"crystal_reports_v4_rpt".as_ptr(),
    mime_type: c"application/x-crystal-reports-v4-rpt".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Seagate Crystal Reports 4.x/5.x Report (.rpt)".as_ptr(),
    extensions: [c"rpt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cs".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"C# source".as_ptr(),
    extensions: [c"cs".as_ptr(), c"csx".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CSMAN_DAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"csman_dat".as_ptr(),
    mime_type: c"application/x-csman-dat".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Amit / Edimax CSMan Router Configuration Data (.dat)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CSO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cso".as_ptr(),
    mime_type: c"application/x-cso".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Compressed ISO (CSO/CISO) disc image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CSPROJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"csproj".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c".NET project config".as_ptr(),
    extensions: [c"csproj".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CSR_DFU2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"csr_dfu2".as_ptr(),
    mime_type: c"application/x-csr-dfu2".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Cambridge Silicon Radio CSR-dfu2 Bluetooth Firmware Image (.dfu)".as_ptr(),
    extensions: [c"dfu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"css".as_ptr(),
    mime_type: c"text/css".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CSS source".as_ptr(),
    extensions: [c"css".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CSV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"csv".as_ptr(),
    mime_type: c"text/csv".as_ptr(),
    group: c"code".as_ptr(),
    description: c"CSV document".as_ptr(),
    extensions: [c"csv".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static CTM2_MAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ctm2_map".as_ptr(),
    mime_type: c"application/x-ctm2-map".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"CTM v2 Tile Map (.ctm)".as_ptr(),
    extensions: [c"ctm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CTOS_RUN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ctos_run".as_ptr(),
    mime_type: c"application/x-ctos-run".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Convergent CTOS / Burroughs BTOS Executable (.run / .sys, 1981) Convergent Technologies".as_ptr(),
    extensions: [c"run".as_ptr(), c"sys".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CTOS_VOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ctos_vol".as_ptr(),
    mime_type: c"application/x-ctos-volume".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Convergent CTOS / Burroughs BTOS Disk Volume (.img / .ima, 1981) Convergent Technologies".as_ptr(),
    extensions: [c"img".as_ptr(), c"ima".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CTRADER_ALGO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ctrader_algo".as_ptr(),
    mime_type: c"application/x-ctrader-algo".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Spotware cTrader / cAlgo Compiled Trading Algorithm (.algo)".as_ptr(),
    extensions: [c"algo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CUBIC_MXM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cubic_mxm".as_ptr(),
    mime_type: c"audio/x-mxm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Cubic Team Tiny XM Module (.mxm)".as_ptr(),
    extensions: [c"mxm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CUBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cubin".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"NVIDIA CUDA binary".as_ptr(),
    extensions: [c"cubin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CUDA_FATBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cuda_fatbin".as_ptr(),
    mime_type: c"application/x-cuda-fatbin".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"NVIDIA CUDA Fatbinary GPU Code Container (.fatbin)".as_ptr(),
    extensions: [c"fatbin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CUPS_IPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cups_ipp".as_ptr(),
    mime_type: c"application/ipp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CUPS Internet Printing Protocol Control Job (c#####)".as_ptr(),
    extensions: [c"ipp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CXL_HELP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cxl_help".as_ptr(),
    mime_type: c"application/x-cxl-help".as_ptr(),
    group: c"document".as_ptr(),
    description: c"C eXtended Library (CXL) Compiled Help Database (.hlp, 1988) Mike Smedley / Innovative Data Concepts".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CYAN_MOHAWK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cyan_mohawk".as_ptr(),
    mime_type: c"application/x-cyan-mohawk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Cyan Worlds Mohawk Resource Archive (.mhk, 1996) Cyan Worlds".as_ptr(),
    extensions: [c"mhk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CYBERLINK_KC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cyberlink_kc".as_ptr(),
    mime_type: c"application/x-cyberlink-kc".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"CyberLink Compiled UI / Script Resource (.kc)".as_ptr(),
    extensions: [c"kc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CYBERPAINT_SEQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cyberpaint_seq".as_ptr(),
    mime_type: c"video/x-cyberpaint-seq".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Atari ST Antic Cyber Paint Sequence Animation (.seq)".as_ptr(),
    extensions: [c"seq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CYBERTRACKER_C64: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cybertracker_c64".as_ptr(),
    mime_type: c"audio/x-cybertracker-c64".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Commodore 64 Noname Cybertracker SID Module (.ct)".as_ptr(),
    extensions: [c"ct".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CYPRESS_CLM_BLOB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cypress_clm_blob".as_ptr(),
    mime_type: c"application/x-cypress-clm-blob".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Cypress / Broadcom Wi-Fi Country Locale Matrix Regulatory Blob (.clm_blob)".as_ptr(),
    extensions: [c"clm_blob".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static CZ_HELP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"cz_help".as_ptr(),
    mime_type: c"application/x-cz-help".as_ptr(),
    group: c"document".as_ptr(),
    description: c"CZ Help File (.hlp)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static D3D_INDEX_BUFFER_IB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"d3d_index_buffer_ib".as_ptr(),
    mime_type: c"application/x-d3d-index-buffer-ib".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Direct3D 32-Bit Triangle Mesh Index Buffer (.ib)".as_ptr(),
    extensions: [c"ib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAGOR_DXP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dagor_dxp".as_ptr(),
    mime_type: c"application/x-dagor-dxp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Gaijin Dagor Engine Texture Pack Archive (.dxp.bin, 2012) Gaijin Entertainment".as_ptr(),
    extensions: [c"dxp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAGOR_GRP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dagor_grp".as_ptr(),
    mime_type: c"application/x-dagor-grp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Gaijin Dagor Engine Game Resource Pack Archive (.grp, 2012) Gaijin Entertainment".as_ptr(),
    extensions: [c"grp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAGOR_VROMFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dagor_vromfs".as_ptr(),
    mime_type: c"application/x-dagor-vromfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Gaijin Dagor Engine Virtual ROM Filesystem Archive (.vromfs.bin, 2012) Gaijin Entertainment".as_ptr(),
    extensions: [c"vromfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAISYDOT_NLQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"daisydot_nlq".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Roy Shiels Daisy-Dot II / III Near Letter Quality Font (.nlq)".as_ptr(),
    extensions: [c"nlq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAN1_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dan1_archive".as_ptr(),
    mime_type: c"application/x-dan1-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DAN1 Character Animation Package (.dan)".as_ptr(),
    extensions: [c"dan".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DARKWAVE_DWP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"darkwave_dwp".as_ptr(),
    mime_type: c"audio/x-darkwave-dwp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ExperimentalScene DarkWave Studio Project (.dwp)".as_ptr(),
    extensions: [c"dwp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DARPA_CGC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"darpa_cgc".as_ptr(),
    mime_type: c"application/x-darpa-cgc".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"DARPA Cyber Grand Challenge DECREE Executable (.cgc, 2014) DARPA".as_ptr(),
    extensions: [c"cgc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dart".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Dart source".as_ptr(),
    extensions: [c"dart".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DART_DSK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dart_dsk".as_ptr(),
    mime_type: c"application/x-dart-dsk".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Dart Desktop State File (.dsk)".as_ptr(),
    extensions: [c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DATABENTO_DBN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"databento_dbn".as_ptr(),
    mime_type: c"application/x-databento-dbn".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Databento Binary Encoding Market Data Stream (.dbn)".as_ptr(),
    extensions: [c"dbn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAVE_LOWE_DLN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dave_lowe_dln".as_ptr(),
    mime_type: c"audio/x-dave-lowe-new".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Dave Lowe New Uncle Art Amiga Module (.dln)".as_ptr(),
    extensions: [c"dln".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAVID_HANNEY_DH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"david_hanney_dh".as_ptr(),
    mime_type: c"audio/x-dh".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"David Hanney Amiga Chiptune Module (.dh)".as_ptr(),
    extensions: [c"dh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAVID_WHITTAKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"david_whittaker".as_ptr(),
    mime_type: c"audio/x-dw".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"David Whittaker Amiga 68000 Music Module (.dw)".as_ptr(),
    extensions: [c"dw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAWESOME_ABYSS2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dawesome_abyss2".as_ptr(),
    mime_type: c"application/x-dawesome-abyss2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Tracktion / Dawesome Abyss 2 Synthesizer Preset (.abyss2) Peter V / Dawesome".as_ptr(),
    extensions: [c"abyss2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DAWESOME_KULT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dawesome_kult".as_ptr(),
    mime_type: c"application/x-dawesome-kult".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Tracktion / Dawesome KULT Synthesizer Preset (.kult) Peter V / Dawesome".as_ptr(),
    extensions: [c"kult".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DB2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"db2".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"IBM DB2 database / backup".as_ptr(),
    extensions: [c"db2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DBASE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dbase".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"dBASE / FoxPro table".as_ptr(),
    extensions: [c"dbf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DBASE_LBL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dbase_lbl".as_ptr(),
    mime_type: c"application/x-dbase-lbl".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Ashton-Tate dBASE IV Label / Report Design File (.lbl)".as_ptr(),
    extensions: [c"lbl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DBG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dbg".as_ptr(),
    mime_type: c"application/x-debug-symbols".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Windows Separate Debug file (.dbg)".as_ptr(),
    extensions: [c"dbg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DBPF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dbpf".as_ptr(),
    mime_type: c"application/x-maxis-dbpf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Maxis / Electronic Arts Database Packed File (.package / .dat)".as_ptr(),
    extensions: [c"package".as_ptr(), c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DCA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dca".as_ptr(),
    mime_type: c"application/x-ibm-dca".as_ptr(),
    group: c"document".as_ptr(),
    description: c"IBM DisplayWrite DCA RFT/FFT (.dca, PRONOM x-fmt/148)".as_ptr(),
    extensions: [c"dca".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DCS_WORLD_EDM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dcs_world_edm".as_ptr(),
    mime_type: c"application/x-dcs-world-edm".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"DCS World / Eagle Dynamics 3D Model (.edm)".as_ptr(),
    extensions: [c"edm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DDI1_BGRA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ddi1_bgra".as_ptr(),
    mime_type: c"application/x-ddi1-bgra".as_ptr(),
    group: c"image".as_ptr(),
    description: c"DDI1 Uncompressed 32-Bit BGRA Bitmap (.bgra)".as_ptr(),
    extensions: [c"bgra".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"deb".as_ptr(),
    mime_type: c"application/vnd.debian.binary-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Debian binary package".as_ptr(),
    extensions: [c"deb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEC_RT11: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dec_rt11".as_ptr(),
    mime_type: c"application/x-dec-rt11".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DEC PDP-11 RT-11 Disk Filesystem Image (.dsk)".as_ptr(),
    extensions: [c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEC_XXDP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dec_xxdp".as_ptr(),
    mime_type: c"application/x-dec-xxdp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DEC PDP-11 XXDP Diagnostic Filesystem Image (.dsk, 1973) Digital Equipment".as_ptr(),
    extensions: [c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEFINICON_DSI32_E32: MagikaTypeInfo = MagikaTypeInfo {
    label: c"definicon_dsi32_e32".as_ptr(),
    mime_type: c"application/x-definicon-dsi32-e32".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Definicon DSI-32 NS32032 Executable (.e32)".as_ptr(),
    extensions: [c"e32".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEFY_DT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"defy_dt".as_ptr(),
    mime_type: c"audio/x-defy-tracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"DeFy AdLib Tracker Module (.dfy, DeFy DT )".as_ptr(),
    extensions: [c"dfy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEGAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"degas".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Atari ST DEGAS image".as_ptr(),
    extensions: [c"pi1".as_ptr(), c"pi2".as_ptr(), c"pi3".as_ptr(), c"pc1".as_ptr(), c"pc2".as_ptr(), c"pc3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DELL_DIAGNOSTICS_MLM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dell_diagnostics_mlm".as_ptr(),
    mime_type: c"application/x-dell-diagnostics-mlm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Dell Modular System Diagnostics Module (.mlm, 1994) Dell".as_ptr(),
    extensions: [c"mlm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DELPHI_CARD_PUBLISHER_LCP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"delphi_card_publisher_lcp".as_ptr(),
    mime_type: c"application/x-delphi-card-publisher-lcp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Borland Delphi TCardDocument / Desktop Card Publisher (.pub, 1995)".as_ptr(),
    extensions: [c"pub".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DELPHI_CDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"delphi_cds".as_ptr(),
    mime_type: c"application/x-delphi-cds".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Borland / Embarcadero Delphi ClientDataSet Binary Packet (.cds / .rds, MIDAS)".as_ptr(),
    extensions: [c"cds".as_ptr(), c"rds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DELPHI_DCU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"delphi_dcu".as_ptr(),
    mime_type: c"application/x-delphi-dcu".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland / Embarcadero Delphi Compiled Unit (.dcu, 1996) Borland".as_ptr(),
    extensions: [c"dcu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DELTA_RPM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"delta_rpm".as_ptr(),
    mime_type: c"application/x-deltarpm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Red Hat / SUSE Delta RPM Package (.drpm, drpm / DLT3)".as_ptr(),
    extensions: [c"drpm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEMOMAKER_FLM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"demomaker_flm".as_ptr(),
    mime_type: c"video/x-demomaker-flm".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Atari ST DemoMaker / Cyber Paint Film Animation (.flm / .seq)".as_ptr(),
    extensions: [c"flm".as_ptr(), c"seq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DESCENT_HOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"descent_hog".as_ptr(),
    mime_type: c"application/x-descent-hog".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Parallax Software Descent HOG Archive (.hog)".as_ptr(),
    extensions: [c"hog".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DESI_DDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"desi_ddf".as_ptr(),
    mime_type: c"application/x-desi-ddf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"DESI Telephone Label Definition File (.ddf) DESI Telephone Labels".as_ptr(),
    extensions: [c"ddf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DESKTOP_TRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"desktop_tracker".as_ptr(),
    mime_type: c"audio/x-dtt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Equivalence Acorn Archimedes Desktop Tracker Module (.dtt)".as_ptr(),
    extensions: [c"dtt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DESQVIEW_HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"desqview_hlp".as_ptr(),
    mime_type: c"application/x-desqview-hlp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Quarterdeck DESQview / DESQview/X Help File (.hlp)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DESY_FPACK_BOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"desy_fpack_bos".as_ptr(),
    mime_type: c"application/x-desy-fpack-bos".as_ptr(),
    group: c"application".as_ptr(),
    description: c"DESY FPACK / BOS Particle Physics Event Stream (.fpack / .bos)".as_ptr(),
    extensions: [c"fpack".as_ptr(), c"bos".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dex".as_ptr(),
    mime_type: c"application/x-android-dex".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Dalvik dex file".as_ptr(),
    extensions: [c"dex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DG_NOVA_RB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dg_nova_rb".as_ptr(),
    mime_type: c"application/x-dg-nova-rb".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Data General Nova Relocatable Binary (.rb, 1969) Data General".as_ptr(),
    extensions: [c"rb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIABLO2_D2S: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diablo2_d2s".as_ptr(),
    mime_type: c"application/x-diablo2-save".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Blizzard North Diablo II Character Save (.d2s, 2000) Blizzard Entertainment".as_ptr(),
    extensions: [c"d2s".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DICOM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dicom".as_ptr(),
    mime_type: c"application/dicom".as_ptr(),
    group: c"image".as_ptr(),
    description: c"DICOM".as_ptr(),
    extensions: [c"dcm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIESEL_GAI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diesel_gai".as_ptr(),
    mime_type: c"application/x-diesel-gai".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Grin / Overkill Diesel Engine Skeletal Animation (.gai)".as_ptr(),
    extensions: [c"gai".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diff".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Diff file".as_ptr(),
    extensions: [c"diff".as_ptr(), c"patch".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DIFFRACPLUS_RAW_DATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diffracplus_raw_data".as_ptr(),
    mime_type: c"application/x-diffracplus-raw-data".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Bruker AXS DIFFRAC.suite / DIFFRACplus X-Ray Diffraction Raw Data (.raw, PRONOM fmt/1120)".as_ptr(),
    extensions: [c"raw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIFSTREAM_SVF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"difstream_svf".as_ptr(),
    mime_type: c"application/x-difstream-svf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DifStream Smart Version Patch (DFS File, .svf, Gentee / DifStream)".as_ptr(),
    extensions: [c"svf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIGITAL_FM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"digital_fm".as_ptr(),
    mime_type: c"audio/x-dfm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Roel Vanhout Digital-FM AdLib Module (.dfm)".as_ptr(),
    extensions: [c"dfm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIGITAL_SONIX_DSC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"digital_sonix_dsc".as_ptr(),
    mime_type: c"audio/x-digital-sonix-dsc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Digital Sonix & Chrome Amiga Module (.dsc)".as_ptr(),
    extensions: [c"dsc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIMPS_EMB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dimps_emb".as_ptr(),
    mime_type: c"application/x-dimps-emb".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Dimps #EMB Texture/Resource Container (.emb, 2008) Dimps".as_ptr(),
    extensions: [c"emb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIRECTORY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"directory".as_ptr(),
    mime_type: c"inode/directory".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"A directory".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIRECTWAVE_DWP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"directwave_dwp".as_ptr(),
    mime_type: c"audio/x-directwave-dwp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Image-Line DirectWave Sampler Program (.dwp)".as_ptr(),
    extensions: [c"dwp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIRK_BIALLUCH_SMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dirk_bialluch_smp".as_ptr(),
    mime_type: c"audio/x-dirk-bialluch-smp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Dirk Bialluch Amiga Sample Bank (.smp)".as_ptr(),
    extensions: [c"smp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DIRK_BIALLUCH_TPU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dirk_bialluch_tpu".as_ptr(),
    mime_type: c"audio/x-dirk-bialluch".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Dirk Bialluch Amiga Music Module (.tpu)".as_ptr(),
    extensions: [c"tpu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DISKDUPE_DDI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diskdupe_ddi".as_ptr(),
    mime_type: c"application/x-diskdupe-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Micro System Designs DiskDupe Floppy Disk Image (.ddi)".as_ptr(),
    extensions: [c"ddi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DISKOVERY_SCAN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"diskovery_scan".as_ptr(),
    mime_type: c"application/x-diskovery-scan".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Diskovery Disk Inventory Snapshot (.diskovery)".as_ptr(),
    extensions: [c"diskovery".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DLINK_AIH0: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dlink_aih0".as_ptr(),
    mime_type: c"application/x-dlink-aih0".as_ptr(),
    group: c"application".as_ptr(),
    description: c"D-Link AIH0 Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DLINK_DEAFBEAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dlink_deafbead".as_ptr(),
    mime_type: c"application/x-dlink-deafbead".as_ptr(),
    group: c"application".as_ptr(),
    description: c"D-Link DEAFBEAD Firmware Archive Container (.deafbead)".as_ptr(),
    extensions: [c"deafbead".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DLINK_FPKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dlink_fpkg".as_ptr(),
    mime_type: c"application/x-dlink-fpkg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"D-Link FPKG / CPKG NetDefend Firewall Firmware Package (.pkg)".as_ptr(),
    extensions: [c"pkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DLINK_MH01: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dlink_mh01".as_ptr(),
    mime_type: c"application/x-dlink-mh01".as_ptr(),
    group: c"application".as_ptr(),
    description: c"D-Link MH01 Encrypted Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DLINK_SHRS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dlink_shrs".as_ptr(),
    mime_type: c"application/x-dlink-shrs".as_ptr(),
    group: c"application".as_ptr(),
    description: c"D-Link SHRS Encrypted Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DLINK_TLV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dlink_tlv".as_ptr(),
    mime_type: c"application/x-dlink-tlv".as_ptr(),
    group: c"application".as_ptr(),
    description: c"D-Link TLV Board-Tagged Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dm".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Dream Maker".as_ptr(),
    extensions: [c"dm".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DMF_XTRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dmf_xtracker".as_ptr(),
    mime_type: c"audio/x-dmf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"D-Lusion X-Tracker Delusion Digital Music File (.dmf)".as_ptr(),
    extensions: [c"dmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dmg".as_ptr(),
    mime_type: c"application/x-apple-diskimage".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple disk image".as_ptr(),
    extensions: [c"dmg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DMS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dms".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Amiga Disk Masher System compressed disk (.dms)".as_ptr(),
    extensions: [c"dms".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DMTF_PLDM_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dmtf_pldm_firmware".as_ptr(),
    mime_type: c"application/x-dmtf-pldm-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"DMTF DSP0267 PLDM Firmware Update Package (.pldm, 2014) DMTF".as_ptr(),
    extensions: [c"pldm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DMU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dmu".as_ptr(),
    mime_type: c"audio/x-dmu".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Softeyes / Thalamus Amiga Digital Mugician Module (.dmu / .mug)".as_ptr(),
    extensions: [c"dmu".as_ptr(), c"mug".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DNA_SEQUENCE_CHROMATOGRAM_FILE_SCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dna_sequence_chromatogram_file_scf".as_ptr(),
    mime_type: c"application/x-dna-sequence-chromatogram-file-scf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Staden Standard Chromatogram Format DNA Sequence (.scf, PRONOM fmt/1554)".as_ptr(),
    extensions: [c"scf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"doc".as_ptr(),
    mime_type: c"application/msword".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Word CDF document".as_ptr(),
    extensions: [c"doc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOCKERFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dockerfile".as_ptr(),
    mime_type: c"text/x-dockerfile".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Dockerfile".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DOCX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"docx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.wordprocessingml.document".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Word 2007+ document".as_ptr(),
    extensions: [c"docx".as_ptr(), c"docm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOS16M_BW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dos16m_bw".as_ptr(),
    mime_type: c"application/x-dos16m-bw".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Rational Systems / Tenberry DOS/16M Protected-Mode BW Executable (.exp / .exe)".as_ptr(),
    extensions: [c"exp".as_ptr(), c"exe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOS_DIM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dos_dim".as_ptr(),
    mime_type: c"application/x-disk-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ray Arachelian Disk IMage Floppy Archive (.dim)".as_ptr(),
    extensions: [c"dim".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOS_HAYES_MDM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dos_hayes_mdm".as_ptr(),
    mime_type: c"application/x-dos-hayes-mdm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"DOS Hayes AT Command Modem Definition File (.mdm)".as_ptr(),
    extensions: [c"mdm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOSMBR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dosmbr".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Master boot record".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOTNET_JIT_PROFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dotnet_jit_profile".as_ptr(),
    mime_type: c"application/x-dotnet-jit-profile".as_ptr(),
    group: c"executable".as_ptr(),
    description: c".NET Runtime Multicore JIT Startup Profile (.profile)".as_ptr(),
    extensions: [c"profile".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DOTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dotx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.wordprocessingml.template".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Office Word 2007 template".as_ptr(),
    extensions: [c"dotx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dpx".as_ptr(),
    mime_type: c"image/x-dpx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"SMPTE 268M Digital Picture Exchange Image (.dpx)".as_ptr(),
    extensions: [c"dpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DRAGONBONES_DBBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dragonbones_dbbin".as_ptr(),
    mime_type: c"application/x-dragonbones-dbbin".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Egret Technology DragonBones Binary Skeleton (.dbbin)".as_ptr(),
    extensions: [c"dbbin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DRUMTRAKER_DTL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"drumtraker_dtl".as_ptr(),
    mime_type: c"audio/x-dtl".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"AdLib DrumTraker Rhythm Module (.dtl)".as_ptr(),
    extensions: [c"dtl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DRUMWARE_K1_BANK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"drumware_k1_bank".as_ptr(),
    mime_type: c"audio/x-drumware-k1".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Drumware Kawai K1 Synthesizer Sound Bank (.k1i, 1988) Drumware".as_ptr(),
    extensions: [c"k1i".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSDIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsdiff".as_ptr(),
    mime_type: c"audio/x-dff".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Philips / Sony Direct Stream Digital Interchange File Format (DSDIFF)".as_ptr(),
    extensions: [c"dff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSF_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsf_audio".as_ptr(),
    mime_type: c"audio/x-dsf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony Direct Stream Digital Stream File (.dsf)".as_ptr(),
    extensions: [c"dsf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSG_ATONCE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsg_atonce".as_ptr(),
    mime_type: c"application/x-atonce-data".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Vortex ATonce PC-AT hard disk image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSK_ACT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsk_act".as_ptr(),
    mime_type: c"application/x-apricot-disk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ACT Apricot disk image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSM_DSIK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsm_dsik".as_ptr(),
    mime_type: c"audio/x-dsm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Carlos Hasan Digital Sound Interface Kit Module (.dsm)".as_ptr(),
    extensions: [c"dsm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSS_DREAMSTATION: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dss_dreamstation".as_ptr(),
    mime_type: c"audio/x-dss".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Audio Simulation DreamStation Synthesizer Song (.dss)".as_ptr(),
    extensions: [c"dss".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSSTORE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsstore".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Application Desktop Services Store".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DSYM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dsym".as_ptr(),
    mime_type: c"audio/x-dsym".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Acorn Archimedes Digital Symphony Tracker Module (.dsym)".as_ptr(),
    extensions: [c"dsym".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DTB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dtb".as_ptr(),
    mime_type: c"application/x-dtb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Flattened Device Tree Blob (.dtb)".as_ptr(),
    extensions: [c"dtb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DTCH_CHART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dtch_chart".as_ptr(),
    mime_type: c"application/x-dtch-chart".as_ptr(),
    group: c"database".as_ptr(),
    description: c"DTCH Song Chart Database (.dtc)".as_ptr(),
    extensions: [c"dtc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dtm".as_ptr(),
    mime_type: c"audio/x-dtm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Softjee Atari Falcon Digital Tracker Module (.dtm)".as_ptr(),
    extensions: [c"dtm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DUCKDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"duckdb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"DuckDB database".as_ptr(),
    extensions: [c"duckdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DURAGER_UV_REAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"durager_uv_real".as_ptr(),
    mime_type: c"application/x-durager-uv-real".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Durager UV Lamp Calibration Record (.real)".as_ptr(),
    extensions: [c"real".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DV_VIDEO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dv_video".as_ptr(),
    mime_type: c"video/x-dv".as_ptr(),
    group: c"video".as_ptr(),
    description: c"IEC 61834 / SMPTE 314M Digital Video DIF Stream (.dv / .dif, 1995)".as_ptr(),
    extensions: [c"dv".as_ptr(), c"dif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DVD_IFO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dvd_ifo".as_ptr(),
    mime_type: c"application/x-dvd-ifo".as_ptr(),
    group: c"video".as_ptr(),
    description: c"DVD-Video VMGI / VTSI Navigation Information (.ifo / .bup)".as_ptr(),
    extensions: [c"ifo".as_ptr(), c"bup".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DWG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dwg".as_ptr(),
    mime_type: c"image/x-dwg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Autocad Drawing".as_ptr(),
    extensions: [c"dwg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DWIN_DGUS3_ICL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dwin_dgus3_icl".as_ptr(),
    mime_type: c"application/x-dwin-dgus3-icl".as_ptr(),
    group: c"image".as_ptr(),
    description: c"DWIN Technology DGUS II/III T5L HMI Icon Library (.icl, 2017) DWIN Technology".as_ptr(),
    extensions: [c"icl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DXA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dxa".as_ptr(),
    mime_type: c"video/x-dxa".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Adventure Soft / ScummVM DXA Compressed Video (.dxa)".as_ptr(),
    extensions: [c"dxa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DXF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dxf".as_ptr(),
    mime_type: c"image/vnd.dxf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Autocad Drawing Exchange Format".as_ptr(),
    extensions: [c"dxf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static DXTK_SPRITEFONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dxtk_spritefont".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Microsoft DirectX Tool Kit SpriteFont (DXTKfont .spritefont)".as_ptr(),
    extensions: [c"spritefont".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static DYNAMIC_STUDIO_DSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"dynamic_studio_dsm".as_ptr(),
    mime_type: c"audio/x-dsm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Dynamic Studio Professional Tracker Module (.dsm)".as_ptr(),
    extensions: [c"dsm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_ABK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_abk".as_ptr(),
    mime_type: c"application/x-ea-abk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Electronic Arts Sports Audio Bank (.abk, 2004) EA Sports".as_ptr(),
    extensions: [c"abk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_BNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_bnk".as_ptr(),
    mime_type: c"audio/x-ea-bnk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Electronic Arts Audio Sample Bank (.bnk, BNKl/BNKb)".as_ptr(),
    extensions: [c"bnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_EBO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_ebo".as_ptr(),
    mime_type: c"application/x-ea-ebo".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Electronic Arts Sports EBO 3D Stadium/Player Mesh (.ebo, 2002) EA Sports".as_ptr(),
    extensions: [c"ebo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_FFN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_ffn".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Electronic Arts FNTF / FNTS / FNTI Game Font (.ffn / .fnt)".as_ptr(),
    extensions: [c"ffn".as_ptr(), c"fnt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_FSH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_fsh".as_ptr(),
    mime_type: c"image/x-ea-fsh".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Electronic Arts FSH / SHPI Shape Image (.fsh / .ssh)".as_ptr(),
    extensions: [c"fsh".as_ptr(), c"ssh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_SCHL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_schl".as_ptr(),
    mime_type: c"audio/x-ea-schl".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Electronic Arts SCHl / SHEN Multimedia Audio/Video Stream (.asf / .exa)".as_ptr(),
    extensions: [c"asf".as_ptr(), c"exa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EA_VIV4_BH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ea_viv4_bh".as_ptr(),
    mime_type: c"application/x-ea-viv4-bh".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Electronic Arts ViV4 BIG Archive Index Header (.bh, 2005) EA Sports".as_ptr(),
    extensions: [c"bh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EARACHE_EA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"earache_ea".as_ptr(),
    mime_type: c"audio/x-ea".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Casper Amiga EarAche Module (.ea)".as_ptr(),
    extensions: [c"ea".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EASE_GLL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ease_gll".as_ptr(),
    mime_type: c"application/x-ease-gll".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"AFMG EASE Generic Loudspeaker Library (.gll)".as_ptr(),
    extensions: [c"gll".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EASTWEST_TCHK_KEY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eastwest_tchk_key".as_ptr(),
    mime_type: c"application/x-eastwest-tchk-key".as_ptr(),
    group: c"application".as_ptr(),
    description: c"EastWest Soundsonline TChk License Key File (.key)".as_ptr(),
    extensions: [c"key".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ECM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ecm".as_ptr(),
    mime_type: c"application/x-ecm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Error Code Modeler CD-ROM Sector Image (.ecm)".as_ptr(),
    extensions: [c"ecm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ECW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ecw".as_ptr(),
    mime_type: c"image/x-ecw".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Enhanced Compression Wavelet Geospatial (.ecw, PRONOM fmt/371, LoC fdd000648)".as_ptr(),
    extensions: [c"ecw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EDF_MESH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"edf_mesh".as_ptr(),
    mime_type: c"application/x-edf-mesh".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"PiBoSo Engine 3D Model (.edf)".as_ptr(),
    extensions: [c"edf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EDILZSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"edilzss".as_ptr(),
    mime_type: c"application/x-edilzss".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"EDI Install / EDI LZSSLib LZSS Packed File".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EDLIB_D00: MagikaTypeInfo = MagikaTypeInfo {
    label: c"edlib_d00".as_ptr(),
    mime_type: c"audio/x-d00".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Vibrants (Jens-Christian Huus) EdLib AdLib FM Audio (.d00)".as_ptr(),
    extensions: [c"d00".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EDLIB_EDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"edlib_edl".as_ptr(),
    mime_type: c"audio/x-edlib-edl".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jens-Christian Huus EdLib Packed OPL2 Module (.edl)".as_ptr(),
    extensions: [c"edl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EGG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"egg".as_ptr(),
    mime_type: c"application/x-egg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ESTsoft EGG compressed archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ELC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"elc".as_ptr(),
    mime_type: c"application/x-elc".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"GNU Emacs Byte-Compiled Lisp (.elc)".as_ptr(),
    extensions: [c"elc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ELF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"elf".as_ptr(),
    mime_type: c"application/x-executable-elf".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"ELF executable".as_ptr(),
    extensions: [c"elf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ELIXIR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"elixir".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Elixir script".as_ptr(),
    extensions: [c"exs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static EMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"emd".as_ptr(),
    mime_type: c"audio/x-emd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ABT Extended MOD Module (.emd)".as_ptr(),
    extensions: [c"emd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"emf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Enhanced Metafile image data".as_ptr(),
    extensions: [c"emf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eml".as_ptr(),
    mime_type: c"message/rfc822".as_ptr(),
    group: c"text".as_ptr(),
    description: c"RFC 822 mail".as_ptr(),
    extensions: [c"eml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static EMPTY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"empty".as_ptr(),
    mime_type: c"inode/x-empty".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"Empty file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EMS_V1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ems_v1".as_ptr(),
    mime_type: c"audio/x-ems-v1".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Milanerny Electronic Music System v1 Amiga Module (.ems)".as_ptr(),
    extensions: [c"ems".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EMS_V6: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ems_v6".as_ptr(),
    mime_type: c"audio/x-ems-v6".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga Electronic Music System v6 Module (.ems)".as_ptr(),
    extensions: [c"ems".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EMTEX_DOT_DRIVER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"emtex_dot_driver".as_ptr(),
    mime_type: c"application/x-emtex-dot".as_ptr(),
    group: c"application".as_ptr(),
    description: c"emTeX Dot-Matrix Printer Driver Table (.dot, 1990) Eberhard Mattes".as_ptr(),
    extensions: [c"dot".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EMTEX_FONT_LIBRARY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"emtex_font_library".as_ptr(),
    mime_type: c"application/x-emtex-fli".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"emTeX Font Library (.fli, 1990) Eberhard Mattes".as_ptr(),
    extensions: [c"fli".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EMTEX_TCP_CODEPAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"emtex_tcp_codepage".as_ptr(),
    mime_type: c"application/x-emtex-tcp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"emTeX TeX Codepage Translation Table (.tcp, 1991) Eberhard Mattes".as_ptr(),
    extensions: [c"tcp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"enc".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Network General Sniffer DOS capture (TRSNIFWR)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENCASE_E01: MagikaTypeInfo = MagikaTypeInfo {
    label: c"encase_e01".as_ptr(),
    mime_type: c"application/x-ewf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"EnCase / Expert Witness Forensic Disk Image (.E01 / .Ex01)".as_ptr(),
    extensions: [c"e01".as_ptr(), c"ex01".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENCORE_ENC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"encore_enc".as_ptr(),
    mime_type: c"application/x-encore".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Passport Designs Encore Musical Notation Score (.enc)".as_ptr(),
    extensions: [c"enc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENDNOTE_ENF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"endnote_enf".as_ptr(),
    mime_type: c"application/x-endnote-enf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Clarivate / Thomson Reuters EndNote Connection/Filter File (.enf / .enz) ISI ResearchSoft".as_ptr(),
    extensions: [c"enf".as_ptr(), c"enz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENSONIQ_PAF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ensoniq_paf".as_ptr(),
    mime_type: c"audio/x-paris".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ensoniq PARIS Digital Audio File (.paf / .fap)".as_ptr(),
    extensions: [c"paf".as_ptr(), c"fap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENTIS_MIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"entis_mio".as_ptr(),
    mime_type: c"audio/x-entis-mio".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Entis MIO Music Interleaved and Orthogonal Transformed Audio (.mio, 2001) Entis / Leshade Entis".as_ptr(),
    extensions: [c"mio".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENVISION: MagikaTypeInfo = MagikaTypeInfo {
    label: c"envision".as_ptr(),
    mime_type: c"application/x-envision-publisher".as_ptr(),
    group: c"document".as_ptr(),
    description: c"EnVision Publisher DTP Document".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ENVISION_SVF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"envision_svf".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Envision Publisher Scalable Vector Font (.svf)".as_ptr(),
    extensions: [c"svf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EO_WEBENGINE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eo_webengine".as_ptr(),
    mime_type: c"application/x-eo-webengine".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Essential Objects EO.WebEngine IPC Cache (.0_1 / .0_1_x64) Essential Objects".as_ptr(),
    extensions: [c"0_1".as_ptr(), c"0_1_x64".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"epd".as_ptr(),
    mime_type: c"application/x-express-publisher".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Power Up Express Publisher Document (.epd)".as_ptr(),
    extensions: [c"epd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EPLAN_EOX_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eplan_eox_db".as_ptr(),
    mime_type: c"application/x-eplan-eox-db".as_ptr(),
    group: c"database".as_ptr(),
    description: c"EPLAN Electric P8 / EEC Engineering Object Database (.eox)".as_ptr(),
    extensions: [c"eox".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EPLYBNDS_MODEL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eplybnds_model".as_ptr(),
    mime_type: c"model/x-eplybnds".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"EPLYBNDS 3D Polygon Bounding Model (EPLYBNDS)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EPS_DOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eps_dos".as_ptr(),
    mime_type: c"application/postscript".as_ptr(),
    group: c"document".as_ptr(),
    description: c"MS-DOS Binary Encapsulated PostScript (.eps)".as_ptr(),
    extensions: [c"eps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EPUB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"epub".as_ptr(),
    mime_type: c"application/epub+zip".as_ptr(),
    group: c"document".as_ptr(),
    description: c"EPUB document".as_ptr(),
    extensions: [c"epub".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ERB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"erb".as_ptr(),
    mime_type: c"text/x-ruby".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Embedded Ruby source".as_ptr(),
    extensions: [c"erb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ERDAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"erdas".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ERDAS Imagine".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ERLANG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"erlang".as_ptr(),
    mime_type: c"text/x-erlang".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Erlang source".as_ptr(),
    extensions: [c"erl".as_ptr(), c"hrl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ESE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ese".as_ptr(),
    mime_type: c"application/x-ms-ese".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"ESE Db".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ESP_PARTTABLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"esp_parttable".as_ptr(),
    mime_type: c"application/x-espressif-partition-table".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Espressif ESP-IDF Flash Partition Table Binary".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ETHEREUM_ERA1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ethereum_era1".as_ptr(),
    mime_type: c"application/x-ethereum-era1".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Ethereum Era1 / Era / E2Store Historical Block Archive (.era1 / .era / .e2s)".as_ptr(),
    extensions: [c"era1".as_ptr(), c"era".as_ptr(), c"e2s".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ETX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"etx".as_ptr(),
    mime_type: c"audio/x-etx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"EasyTrax Module (.etx)".as_ptr(),
    extensions: [c"etx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EUPHONY_EUP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"euphony_eup".as_ptr(),
    mime_type: c"audio/x-euphony".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Fujitsu FM Towns Euphony Song Module (.eup)".as_ptr(),
    extensions: [c"eup".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EUREKALOG_EL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eurekalog_el".as_ptr(),
    mime_type: c"application/x-eurekalog-el".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Neos Eureka S.r.l. EurekaLog Delphi/C++Builder Exception Log (.el, 2011) EurekaLog".as_ptr(),
    extensions: [c"el".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EVT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"evt".as_ptr(),
    mime_type: c"application/x-ms-evt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows NT/2000/XP Legacy Event Log (.evt)".as_ptr(),
    extensions: [c"evt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EVTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"evtx".as_ptr(),
    mime_type: c"application/x-ms-evtx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows XML Event Log (.evtx)".as_ptr(),
    extensions: [c"evtx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EVY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"evy".as_ptr(),
    mime_type: c"application/x-envoy".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Novell / Tumbleweed Envoy Document (.evy)".as_ptr(),
    extensions: [c"evy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EXBN_EBN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"exbn_ebn".as_ptr(),
    mime_type: c"application/x-exbn-ebn".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"EXBN Binary Resource Container (.ebn)".as_ptr(),
    extensions: [c"ebn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EXIV2_EXV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"exiv2_exv".as_ptr(),
    mime_type: c"image/x-exv".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Exiv2 Metadata Sidecar Container (.exv, 2004) Andreas Huggel".as_ptr(),
    extensions: [c"exv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EXODUS_SECO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"exodus_seco".as_ptr(),
    mime_type: c"application/x-exodus-seco".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Exodus Wallet SECO Encrypted Container (.seco)".as_ptr(),
    extensions: [c"seco".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EXOTIC_XAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"exotic_xad".as_ptr(),
    mime_type: c"audio/x-xad".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Riven the Mage eXotic AdLib Module (.xad)".as_ptr(),
    extensions: [c"xad".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EXR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"exr".as_ptr(),
    mime_type: c"image/x-exr".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ILM OpenEXR high dynamic range image (.exr)".as_ptr(),
    extensions: [c"exr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EXT4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ext4".as_ptr(),
    mime_type: c"application/x-raw-disk-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Linux Extended Filesystem (Ext2/Ext3/Ext4)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EZART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ezart".as_ptr(),
    mime_type: c"image/x-ezart".as_ptr(),
    group: c"image".as_ptr(),
    description: c"EZ-Art Professional Graphic (.eza)".as_ptr(),
    extensions: [c"eza".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static EZNEC_EZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"eznec_ez".as_ptr(),
    mime_type: c"application/x-eznec-ez".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"EZNEC Antenna Modeling Description (.ez, Roy Lewallen W7EL)".as_ptr(),
    extensions: [c"ez".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FABYLON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fabylon".as_ptr(),
    mime_type: c"application/x-fabylon".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Fabylon electronic book".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FACEGEN_TRI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"facegen_tri".as_ptr(),
    mime_type: c"application/x-facegen-tri".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Singular Inversions FaceGen / Creation Engine TRI Head Morph Mesh (.tri, 2003) Singular Inversions".as_ptr(),
    extensions: [c"tri".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FAIRYGUI_FUI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fairygui_fui".as_ptr(),
    mime_type: c"application/x-fairygui-fui".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"FairyGUI Binary UI Asset Package (.fui)".as_ptr(),
    extensions: [c"fui".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FALCOM_MI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"falcom_mi".as_ptr(),
    mime_type: c"application/x-falcom-mi".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Nihon Falcom Trails / Kuro Engine Binary JSON Model Info (.mi)".as_ptr(),
    extensions: [c"mi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FANUC_PMC_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fanuc_pmc_patch".as_ptr(),
    mime_type: c"application/x-fanuc-pmc-patch".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Fanuc PMC CNC Machine Controller Patch (.bfd/.kcf)".as_ptr(),
    extensions: [c"bfd".as_ptr(), c"kcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FARBRAUSCH_V2M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"farbrausch_v2m".as_ptr(),
    mime_type: c"audio/x-v2m".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Farbrausch V2 Synthesizer Module (.v2m)".as_ptr(),
    extensions: [c"v2m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FASTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fasta".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"FASTA sequence data".as_ptr(),
    extensions: [c"fa".as_ptr(), c"fasta".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static FASTGRAPH_FGI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fastgraph_fgi".as_ptr(),
    mime_type: c"application/x-fastgraph-fgi".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Fastgraph/Image Resource Library (.fgi, 1991) Ted Gruber Software".as_ptr(),
    extensions: [c"fgi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FASTGRAPH_PPR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fastgraph_ppr".as_ptr(),
    mime_type: c"image/x-fastgraph-ppr".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Fastgraph Packed Pixel Run / Young Picasso Bitmap (.yp / .ppr, 1991) Ted Gruber Software / Blueview Software".as_ptr(),
    extensions: [c"yp".as_ptr(), c"ppr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FASTGRAPH_SPR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fastgraph_spr".as_ptr(),
    mime_type: c"image/x-fastgraph-spr".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Fastgraph Standard Pixel Run Bitmap (.prf / .spr, 1991) Ted Gruber Software".as_ptr(),
    extensions: [c"prf".as_ptr(), c"spr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FASTQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fastq".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"FASTQ sequence data".as_ptr(),
    extensions: [c"fastq".as_ptr(), c"fq".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static FBA_FS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fba_fs".as_ptr(),
    mime_type: c"application/x-fba-fs".as_ptr(),
    group: c"application".as_ptr(),
    description: c"FinalBurn Alpha / FBNeo Emulator Savestate (.fs, 2002) Team FBAlpha".as_ptr(),
    extensions: [c"fs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FBCHUNKS_NGP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fbchunks_ngp".as_ptr(),
    mime_type: c"application/x-fbchunks-ngp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"FBCHUNKS Chunked Package (.ngp)".as_ptr(),
    extensions: [c"ngp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FBS_TIMETABLE_FPL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fbs_timetable_fpl".as_ptr(),
    mime_type: c"application/x-fbs-timetable-fpl".as_ptr(),
    group: c"database".as_ptr(),
    description: c"iRFP FBS Railway Timetable (.fpl)".as_ptr(),
    extensions: [c"fpl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FBX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fbx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Autodesk FBX".as_ptr(),
    extensions: [c"fbx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fds".as_ptr(),
    mime_type: c"application/x-fds-rom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Famicom Disk System Disk Image (.fds)".as_ptr(),
    extensions: [c"fds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIELDLOG_FL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fieldlog_fl".as_ptr(),
    mime_type: c"application/x-fieldlog-fl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NetManila / Deventec FieldLog Binary Diagnostic Trace (.fl)".as_ptr(),
    extensions: [c"fl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fif".as_ptr(),
    mime_type: c"image/x-fif".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Iterated Systems Fractal Image Format (.fif)".as_ptr(),
    extensions: [c"fif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIFA_FIFAMOD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fifa_fifamod".as_ptr(),
    mime_type: c"application/x-fifa-fifamod".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"FIFA Editor Tool Mod Package (.fifamod)".as_ptr(),
    extensions: [c"fifamod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIGHTER_MAKER_KGT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fighter_maker_kgt".as_ptr(),
    mime_type: c"application/x-fighter-maker-kgt".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ASCII / Enterbrain 2D Fighter Maker 95 Game Data (.kgt / .bg, 1999) Enterbrain".as_ptr(),
    extensions: [c"kgt".as_ptr(), c"bg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIGMA_FIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"figma_fig".as_ptr(),
    mime_type: c"application/x-figma".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Figma Binary Kiwi Canvas Document (.fig)".as_ptr(),
    extensions: [c"fig".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FILAMENT_FILAMAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"filament_filamat".as_ptr(),
    mime_type: c"application/x-filament-material".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Google Filament Compiled Material Package (.filamat)".as_ptr(),
    extensions: [c"filamat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FILEMAKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"filemaker".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"FileMaker database".as_ptr(),
    extensions: [c"fmp12".as_ptr(), c"fp7".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FINEPRINT_FP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fineprint_fp".as_ptr(),
    mime_type: c"application/x-fineprint-fp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"FinePrint Saved Output (.fp, PRONOM fmt/1396)".as_ptr(),
    extensions: [c"fp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIO_VERIFY_STATE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fio_verify_state".as_ptr(),
    mime_type: c"application/x-fio-verify-state".as_ptr(),
    group: c"application".as_ptr(),
    description: c"fio (Flexible I/O Tester) Verify State (.state)".as_ptr(),
    extensions: [c"state".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIREBIRD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"firebird".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Firebird / InterBase database".as_ptr(),
    extensions: [c"fdb".as_ptr(), c"gdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FIT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fit".as_ptr(),
    mime_type: c"application/vnd.garmin.fit".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Garmin Flexible and Interoperable Data Transfer (.fit)".as_ptr(),
    extensions: [c"fit".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FITS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fits".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"FITS astronomy data".as_ptr(),
    extensions: [c"fit".as_ptr(), c"fits".as_ptr(), c"fts".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FL_STUDIO_FLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fl_studio_flp".as_ptr(),
    mime_type: c"application/x-flp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Image-Line FruityLoops / FL Studio Project (.flp, FLhd)".as_ptr(),
    extensions: [c"flp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flac".as_ptr(),
    mime_type: c"audio/flac".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"FLAC audio bitstream data".as_ptr(),
    extensions: [c"flac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLASH_SOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flash_sol".as_ptr(),
    mime_type: c"application/x-flash-sol".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Adobe Flash Player Local Shared Object (.sol, 2002) Macromedia / Adobe".as_ptr(),
    extensions: [c"sol".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLATGEOBUF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flatgeobuf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"FlatGeobuf".as_ptr(),
    extensions: [c"fgb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLATPAK_BUNDLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flatpak_bundle".as_ptr(),
    mime_type: c"application/vnd.flatpak".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Flatpak Single-File Application Bundle (.flatpak)".as_ptr(),
    extensions: [c"flatpak".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLIGHTSIM_NAME_AIR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flightsim_name_air".as_ptr(),
    mime_type: c"application/x-flightsim-name-air".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Flight Simulator Tagged Aircraft Descriptor (.air)".as_ptr(),
    extensions: [c"air".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLIR_FPF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flir_fpf".as_ptr(),
    mime_type: c"image/x-flir-fpf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Teledyne FLIR Public Image Format Thermal Radiometric Image (.fpf)".as_ptr(),
    extensions: [c"fpf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLMOBILE_FLMRACK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flmobile_flmrack".as_ptr(),
    mime_type: c"application/x-flmobile-flmrack".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Image-Line FL Studio Mobile Effect/Instrument Rack (.flmrack) Image-Line".as_ptr(),
    extensions: [c"flmrack".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flog".as_ptr(),
    mime_type: c"image/x-flogiciels".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Flogiciels / Loriciels image (.cpt/.lcr)".as_ptr(),
    extensions: [c"cpt".as_ptr(), c"lcr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLUENT_INTERP_IP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fluent_interp_ip".as_ptr(),
    mime_type: c"application/x-fluent-interp-ip".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ANSYS Fluent Interpolation Field Data (.ip)".as_ptr(),
    extensions: [c"ip".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FLV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"flv".as_ptr(),
    mime_type: c"video/x-flv".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Flash Video".as_ptr(),
    extensions: [c"flv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FM_FMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fm_fmf".as_ptr(),
    mime_type: c"application/x-fm-fmf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sports Interactive Football Manager Data Archive (.fmf, Sports Interactive)".as_ptr(),
    extensions: [c"fmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FM_KINGTRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fm_kingtracker".as_ptr(),
    mime_type: c"audio/x-fmk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"FM-Kingtracker AdLib Module (.fmk)".as_ptr(),
    extensions: [c"fmk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FM_TRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fm_tracker".as_ptr(),
    mime_type: c"audio/x-fmt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"FM Tracker AdLib Module (.fmt)".as_ptr(),
    extensions: [c"fmt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FMC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fmc".as_ptr(),
    mime_type: c"audio/x-fmc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Faust Music Creator AdLib Module (.sng)".as_ptr(),
    extensions: [c"sng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FMOD_FEV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fmod_fev".as_ptr(),
    mime_type: c"audio/x-fmod-fev".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Firelight Technologies FMOD Designer Compiled Event Project (.fev, 2006) Firelight Technologies".as_ptr(),
    extensions: [c"fev".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FMP_M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fmp_m".as_ptr(),
    mime_type: c"audio/x-fmp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Guu PC-98 FMP Music Driver Module (.m)".as_ptr(),
    extensions: [c"m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FMTOWNS_EUPHONY_PMB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fmtowns_euphony_pmb".as_ptr(),
    mime_type: c"audio/x-fmtowns-euphony-pmb".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Fujitsu FM Towns Euphony PCM Instrument Bank (.pmb)".as_ptr(),
    extensions: [c"pmb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fog".as_ptr(),
    mime_type: c"application/x-fontographer".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Altsys / Macromedia Fontographer Database (.fog)".as_ptr(),
    extensions: [c"fog".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FONTCONFIG_CACHE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fontconfig_cache".as_ptr(),
    mime_type: c"application/x-fontconfig-cache".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Freedesktop.org Fontconfig Binary Font Cache (.cache-*)".as_ptr(),
    extensions: [c"cache".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FORTRAN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fortran".as_ptr(),
    mime_type: c"text/x-fortran".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Fortran".as_ptr(),
    extensions: [c"f90".as_ptr(), c"f95".as_ptr(), c"f03".as_ptr(), c"F90".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static FOURDOS_HELP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fourdos_help".as_ptr(),
    mime_type: c"application/x-4dos-help".as_ptr(),
    group: c"document".as_ptr(),
    description: c"JP Software 4DOS Compressed Help Database (.hlp, 1991) JP Software".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FOXPRO_FORM_SCT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"foxpro_form_sct".as_ptr(),
    mime_type: c"application/x-foxpro-form-sct".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Visual FoxPro 3.0 Form/Class Memo (.sct / .vct)".as_ptr(),
    extensions: [c"sct".as_ptr(), c"vct".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fpx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Flashpix".as_ptr(),
    extensions: [c"fpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FREEARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"freearc".as_ptr(),
    mime_type: c"application/x-freearc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"FreeArc Compressed Archive (.arc)".as_ptr(),
    extensions: [c"arc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FREEBASIC_HF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"freebasic_hf".as_ptr(),
    mime_type: c"application/x-freebasic-hf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Borland Turbo Vision / FreeVision Compiled Help File (.hlp / .hf, 1990) Borland".as_ptr(),
    extensions: [c"hlp".as_ptr(), c"hf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FREEDOM_FORCE_SAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"freedom_force_sav".as_ptr(),
    mime_type: c"application/x-freedom-force-save".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Irrational Games Freedom Force Save Game (.dat)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FROMSOFT_BINDER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fromsoft_binder".as_ptr(),
    mime_type: c"application/x-fromsoft-binder".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"FromSoftware Binder Archive / Header (.bnd / .bhf / .tpf, 2009) FromSoftware".as_ptr(),
    extensions: [c"bnd".as_ptr(), c"bhf".as_ptr(), c"tpf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FROMSOFT_DCX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fromsoft_dcx".as_ptr(),
    mime_type: c"application/x-fromsoft-dcx".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"FromSoftware DCX Compressed Asset Container (.dcx, 2009) FromSoftware".as_ptr(),
    extensions: [c"dcx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FROMSOFT_FLVER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fromsoft_flver".as_ptr(),
    mime_type: c"model/x-fromsoft-flver".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"FromSoftware FLVER 3D Model (.flver, 2009) FromSoftware".as_ptr(),
    extensions: [c"flver".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FROMSOFT_TAE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fromsoft_tae".as_ptr(),
    mime_type: c"application/x-fromsoft-tae".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"FromSoftware TimeAct Animation Event Table (.tae, 2009) FromSoftware".as_ptr(),
    extensions: [c"tae".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FROSTY_FBMOD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"frosty_fbmod".as_ptr(),
    mime_type: c"application/x-frosty-fbmod".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Frosty Toolsuite EA Frostbite Engine Mod Package (.fbmod)".as_ptr(),
    extensions: [c"fbmod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FSECURE_FS02_MF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fsecure_fs02_mf".as_ptr(),
    mime_type: c"application/x-fsecure-manifest".as_ptr(),
    group: c"application".as_ptr(),
    description: c"F-Secure Signed Update Manifest (FS02, .mf)".as_ptr(),
    extensions: [c"mf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FTK_AD1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ftk_ad1".as_ptr(),
    mime_type: c"application/x-ftk-ad1".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"AccessData Forensic Toolkit (FTK) Logical Evidence Image (.ad1, ADSEGMENTEDFILE)".as_ptr(),
    extensions: [c"ad1".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ftm".as_ptr(),
    mime_type: c"audio/x-ftm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NES FamiTracker Module (.ftm)".as_ptr(),
    extensions: [c"ftm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FTMN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ftmn".as_ptr(),
    mime_type: c"audio/x-ftm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jörg Schmidt / Maxon Amiga Face The Music 8-Channel Module (.ftm)".as_ptr(),
    extensions: [c"ftm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FULLWRITE_DOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"fullwrite_doc".as_ptr(),
    mime_type: c"application/x-fullwrite".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Ashton-Tate / Akimbo FullWrite Professional 1.x/2.x Macintosh Document (.fw / .fwp)".as_ptr(),
    extensions: [c"fw".as_ptr(), c"fwp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FUNKTRACKER_FNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"funktracker_fnk".as_ptr(),
    mime_type: c"audio/x-fnk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jason Nunn Funktracker Module (.fnk)".as_ptr(),
    extensions: [c"fnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static FUTURE_COMPOSER_BSI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"future_composer_bsi".as_ptr(),
    mime_type: c"audio/x-future-composer-bsi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Superions Future Composer BSI Amiga Module (.bsi)".as_ptr(),
    extensions: [c"bsi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static G9B_BITMAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"g9b_bitmap".as_ptr(),
    mime_type: c"image/x-g9b".as_ptr(),
    group: c"document".as_ptr(),
    description: c"G9B MSX2+ V9990 Graphics Format Bitmap (.g9b, PRONOM fmt/1787)".as_ptr(),
    extensions: [c"g9b".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GALAXY_MUSIC_MUSE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"galaxy_music_muse".as_ptr(),
    mime_type: c"audio/x-galaxy-muse".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Carlo Vogelsang Galaxy Music System Audio (MUSE)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GAME_MUSIC_CREATOR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"game_music_creator".as_ptr(),
    mime_type: c"audio/x-game-music-creator".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Andreas Tadic Game Music Creator Amiga Module (.gmc)".as_ptr(),
    extensions: [c"gmc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GAMEBOY_GBR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gameboy_gbr".as_ptr(),
    mime_type: c"audio/x-gbr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Game Boy Ripped Sound Format (.gbr)".as_ptr(),
    extensions: [c"gbr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GAMEBOY_GBS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gameboy_gbs".as_ptr(),
    mime_type: c"audio/x-gbs".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo Game Boy Sound System Chiptune Rip (.gbs)".as_ptr(),
    extensions: [c"gbs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GAMEMAKER_GMK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gamemaker_gmk".as_ptr(),
    mime_type: c"application/x-gamemaker-gmk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Mark Overmars / YoYo Games GameMaker Legacy Project (.gm6 / .gmk / .gm81, 2004) YoYo Games".as_ptr(),
    extensions: [c"gm6".as_ptr(), c"gmk".as_ptr(), c"gm81".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GARMIN_GMN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"garmin_gmn".as_ptr(),
    mime_type: c"application/x-garmin-gmn".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Garmin Track Log File (.gmn, PRONOM fmt/1679)".as_ptr(),
    extensions: [c"gmn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GBA_GBFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gba_gbfs".as_ptr(),
    mime_type: c"application/x-gba-gbfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Damian Yerrick Pineight Game Boy Advance ROM File System (.gbfs)".as_ptr(),
    extensions: [c"gbfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GCA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gca".as_ptr(),
    mime_type: c"application/x-gca-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"GCA / DGCA compressed archive (.gca/.dgc)".as_ptr(),
    extensions: [c"gca".as_ptr(), c"dgc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GDM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gdm".as_ptr(),
    mime_type: c"audio/x-gdm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bells, Whistles, and Sound Boards General Digital Music Module (.gdm)".as_ptr(),
    extensions: [c"gdm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEA_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gea_archive".as_ptr(),
    mime_type: c"application/x-gea-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"GEA Installer Uninstall Package (.gea)".as_ptr(),
    extensions: [c"gea".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GECKO_STARTUPCACHE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gecko_startupcache".as_ptr(),
    mime_type: c"application/x-gecko-startupcache".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Mozilla Firefox / Gecko Startup Cache (startupcache)".as_ptr(),
    extensions: [c"little".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEM_VDI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gem_vdi".as_ptr(),
    mime_type: c"image/x-gem".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Digital Research GEM VDI Metafile (.gem)".as_ptr(),
    extensions: [c"gem".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEMFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gemfile".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gemfile file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GEMSPEC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gemspec".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gemspec file".as_ptr(),
    extensions: [c"gemspec".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GENESIS_GYM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"genesis_gym".as_ptr(),
    mime_type: c"audio/x-gym".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sega Genesis YM2612 Audio Log (.gym)".as_ptr(),
    extensions: [c"gym".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GENH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"genh".as_ptr(),
    mime_type: c"audio/x-genh".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"vgmstream Generic Headered Audio Stream (.genh)".as_ptr(),
    extensions: [c"genh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEO_GTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"geo_gtx".as_ptr(),
    mime_type: c"application/x-geo-gtx".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"GEO 3D Voxel Chunk (.gtx)".as_ptr(),
    extensions: [c"gtx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEOJSON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"geojson".as_ptr(),
    mime_type: c"application/geo+json".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"GeoJSON".as_ptr(),
    extensions: [c"geojson".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GEOPACKAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"geopackage".as_ptr(),
    mime_type: c"application/geopackage+sqlite3".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"OGC GeoPackage".as_ptr(),
    extensions: [c"gpkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEOSOFT_GX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"geosoft_gx".as_ptr(),
    mime_type: c"application/x-geosoft-gx".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Geosoft Oasis montaj Compiled GX Executable (.gx)".as_ptr(),
    extensions: [c"gx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GEOWORKS_GEOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"geoworks_geos".as_ptr(),
    mime_type: c"application/x-geoworks-geos".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"PC/GEOS Geoworks GeoFile Executable / Document (.geo / .vm)".as_ptr(),
    extensions: [c"geo".as_ptr(), c"vm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GETTEXT_MO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gettext_mo".as_ptr(),
    mime_type: c"application/x-gettext-translation".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GNU gettext Machine Object Compiled Translation Catalog (.mo)".as_ptr(),
    extensions: [c"mo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GGML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ggml".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Legacy GGML model".as_ptr(),
    extensions: [c"ggml".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GGUF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gguf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"model".as_ptr(),
    description: c"GGUF".as_ptr(),
    extensions: [c"gguf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GHC_HI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ghc_hi".as_ptr(),
    mime_type: c"application/x-ghc-interface".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"GHC Team Glasgow Haskell Compiler Interface File (.hi / .dyn_hi / .p_hi)".as_ptr(),
    extensions: [c"hi".as_ptr(), c"dyn_hi".as_ptr(), c"p_hi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GHIDRA_BF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ghidra_bf".as_ptr(),
    mime_type: c"application/x-ghidra-bf".as_ptr(),
    group: c"database".as_ptr(),
    description: c"NSA Ghidra Unpacked Indexed BufferFile Database (.bf, /01*49,+)".as_ptr(),
    extensions: [c"bf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GIANTS_GDM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"giants_gdm".as_ptr(),
    mime_type: c"image/x-giants-gdm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"GIANTS Engine Compressed Foliage Density Map (.gdm, 2014) GIANTS Software".as_ptr(),
    extensions: [c"gdm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GIANTS_GRLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"giants_grle".as_ptr(),
    mime_type: c"image/x-giants-grle".as_ptr(),
    group: c"image".as_ptr(),
    description: c"GIANTS Engine Run-Length Encoded Foliage / Terrain Map (.grle, 2008) GIANTS Software".as_ptr(),
    extensions: [c"grle".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gif".as_ptr(),
    mime_type: c"image/gif".as_ptr(),
    group: c"image".as_ptr(),
    description: c"GIF image data".as_ptr(),
    extensions: [c"gif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GIT_INDEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"git_index".as_ptr(),
    mime_type: c"application/x-git-index".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Git Working Tree Staging Index (.git/index, DIRC)".as_ptr(),
    extensions: [c"git".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GIT_PACK_IDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"git_pack_idx".as_ptr(),
    mime_type: c"application/x-git-pack-idx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Git Packfile v2 Fanout Index (.idx, \\\\xFFtOc)".as_ptr(),
    extensions: [c"idx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GIT_PACK_REV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"git_pack_rev".as_ptr(),
    mime_type: c"application/x-git-pack-rev".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Git Packfile Reverse Index (.rev, 2021) Git Project".as_ptr(),
    extensions: [c"rev".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GITATTRIBUTES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gitattributes".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gitattributes file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GITMODULES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gitmodules".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gitmodules file".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GLIBC_LD_SO_CACHE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"glibc_ld_so_cache".as_ptr(),
    mime_type: c"application/x-glibc-ld-so-cache".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GNU C Library Dynamic Linker Cache (ld.so.cache, glibc-ld.so.cache1.1)".as_ptr(),
    extensions: [c"cache".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GLIDE_3DF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"glide_3df".as_ptr(),
    mime_type: c"image/x-3dfx-3df".as_ptr(),
    group: c"image".as_ptr(),
    description: c"3dfx Glide Texture File (.3df)".as_ptr(),
    extensions: [c"3df".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GLTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gltf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"glTF".as_ptr(),
    extensions: [c"glb".as_ptr(), c"gltf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GLUEMON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gluemon".as_ptr(),
    mime_type: c"audio/x-gluemon".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"GlueMon Amiga Chiptune Module (.glue)".as_ptr(),
    extensions: [c"glue".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GLULX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"glulx".as_ptr(),
    mime_type: c"application/x-glulx".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Glulx Virtual Machine Story File (.ulx)".as_ptr(),
    extensions: [c"ulx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GMA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gma".as_ptr(),
    mime_type: c"application/x-garrysmod-gma".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Facepunch Garry's Mod Addon Archive (.gma)".as_ptr(),
    extensions: [c"gma".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gml".as_ptr(),
    mime_type: c"application/gml+xml".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"Geography Markup Language".as_ptr(),
    extensions: [c"gml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GNOME_KEYRING: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gnome_keyring".as_ptr(),
    mime_type: c"application/x-gnome-keyring".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GNOME Keyring Locked Credential Store (.keyring)".as_ptr(),
    extensions: [c"keyring".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GNU_LOCATE_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gnu_locate_db".as_ptr(),
    mime_type: c"application/x-locate-db".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GNU findutils locate File Name Database (LOCATE02)".as_ptr(),
    extensions: [c"locatedb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GNUNET_GND: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gnunet_gnd".as_ptr(),
    mime_type: c"application/x-gnunet-directory".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GNUnet Directory Listing (.gnd, \\\\x89GND\\\\r\\\\n\\\\x1A\\\\n)".as_ptr(),
    extensions: [c"gnd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"go".as_ptr(),
    mime_type: c"text/x-golang".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Golang source".as_ptr(),
    extensions: [c"go".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GO_TELEMETRY_COUNT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"go_telemetry_count".as_ptr(),
    mime_type: c"application/x-go-telemetry-count".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Go Runtime Telemetry Counter File (.count, 2023) Go Authors".as_ptr(),
    extensions: [c"count".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GOATTRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"goattracker".as_ptr(),
    mime_type: c"audio/x-goattracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Commodore 64 GoatTracker Chiptune Song / Instrument (.sng / .ins)".as_ptr(),
    extensions: [c"sng".as_ptr(), c"ins".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GOBJECT_TYPELIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gobject_typelib".as_ptr(),
    mime_type: c"application/x-gobject-typelib".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GNOME GObject Introspection Compiled Type Library (.typelib)".as_ptr(),
    extensions: [c"typelib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GODOT_GDEC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"godot_gdec".as_ptr(),
    mime_type: c"application/x-godot-gdec".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Godot Engine Encrypted File (GDEC, FileAccessEncrypted)".as_ptr(),
    extensions: [c"ntr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GODOT_PCK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"godot_pck".as_ptr(),
    mime_type: c"application/x-godot-pck".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Godot Engine Resource Pack (.pck)".as_ptr(),
    extensions: [c"pck".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GODOT_RSRC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"godot_rsrc".as_ptr(),
    mime_type: c"application/x-godot-rsrc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Godot Engine Binary Resource / Scene (.res / .scn, 2014) Godot Engine".as_ptr(),
    extensions: [c"res".as_ptr(), c"scn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GOFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"goff".as_ptr(),
    mime_type: c"application/x-ibm-goff".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"IBM z/OS Generalized Object File Format (GOFF)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GOLDSRC_WAD3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"goldsrc_wad3".as_ptr(),
    mime_type: c"application/x-goldsrc-wad3".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Valve GoldSrc / Half-Life Texture WAD3 Archive (.wad, 1998) Valve".as_ptr(),
    extensions: [c"wad".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GPG_TRUSTDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gpg_trustdb".as_ptr(),
    mime_type: c"application/x-gnupg-trustdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"GNU Privacy Guard Key Trust Database (trustdb.gpg)".as_ptr(),
    extensions: [c"gpg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GPO_REGISTRY_POL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gpo_registry_pol".as_ptr(),
    mime_type: c"application/x-ms-registry-pol".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Group Policy Administrative Template Policy (registry.pol, PReg)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gpx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"XML document".as_ptr(),
    extensions: [c"gpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GRADLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gradle".as_ptr(),
    mime_type: c"text/x-groovy".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Gradle source".as_ptr(),
    extensions: [c"gradle".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GRAFX2_PKM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"grafx2_pkm".as_ptr(),
    mime_type: c"image/x-grafx2-pkm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sunset Design GrafX2 PKM Paletted Bitmap (.pkm)".as_ptr(),
    extensions: [c"pkm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GRAOUMF_GT2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"graoumf_gt2".as_ptr(),
    mime_type: c"audio/x-gt2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Laurent de Soras Graoumf Tracker 1/2 Module (.gtk / .gt2)".as_ptr(),
    extensions: [c"gtk".as_ptr(), c"gt2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GRIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"grib".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"GRIB meteorological data".as_ptr(),
    extensions: [c"grb".as_ptr(), c"grb2".as_ptr(), c"grib".as_ptr(), c"grib2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GROOVY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"groovy".as_ptr(),
    mime_type: c"text/x-groovy".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Groovy source".as_ptr(),
    extensions: [c"groovy".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static GSDD_GDD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gsdd_gdd".as_ptr(),
    mime_type: c"application/x-gsdd-gdd".as_ptr(),
    group: c"database".as_ptr(),
    description: c"GSDD Display Descriptor Database (.gdd)".as_ptr(),
    extensions: [c"gdd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GSSI_DZT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gssi_dzt".as_ptr(),
    mime_type: c"application/x-gssi-dzt".as_ptr(),
    group: c"document".as_ptr(),
    description: c"GSSI SIR-10 RADAN Ground Penetrating Radar Data (.dzt, PRONOM fmt/362)".as_ptr(),
    extensions: [c"dzt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GSYM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gsym".as_ptr(),
    mime_type: c"application/x-llvm-gsym".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"LLVM GSYM Debug Symbol File".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GT_GAME_SYSTEMS_DUX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gt_game_systems_dux".as_ptr(),
    mime_type: c"audio/x-gt-game-systems-dux".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"GT Game Systems Amiga Music Module (.dux)".as_ptr(),
    extensions: [c"dux".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GTA_IFP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gta_ifp".as_ptr(),
    mime_type: c"application/x-gta-ifp".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Rockstar Games GTA RenderWare Animation Package (.ifp, 2001) Rockstar Games".as_ptr(),
    extensions: [c"ifp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GUEMAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"guemap".as_ptr(),
    mime_type: c"application/x-guemap".as_ptr(),
    group: c"document".as_ptr(),
    description: c"GUEmap Interactive Fiction Map (.gmp)".as_ptr(),
    extensions: [c"gmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GUITAR_PRO_GPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"guitar_pro_gpx".as_ptr(),
    mime_type: c"application/x-guitar-pro-gpx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Arobas Music Guitar Pro 6 Tablature Container (.gpx)".as_ptr(),
    extensions: [c"gpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GUS_PAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gus_pat".as_ptr(),
    mime_type: c"audio/x-gus-patch".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Gravis Ultrasound GF1 Instrument Patch (.pat)".as_ptr(),
    extensions: [c"pat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GX1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gx1".as_ptr(),
    mime_type: c"image/x-gx1".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Brightbill-Roberts Show Partner GX1 Graphics Image (.gx1)".as_ptr(),
    extensions: [c"gx1".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GX2_BITMAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gx2_bitmap".as_ptr(),
    mime_type: c"image/x-gx2".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Show Partner GX2 Graphics File (.gx2, PRONOM fmt/1789)".as_ptr(),
    extensions: [c"gx2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static GZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"gzip".as_ptr(),
    mime_type: c"application/gzip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"gzip compressed data".as_ptr(),
    extensions: [c"gz".as_ptr(), c"gzip".as_ptr(), c"tgz".as_ptr(), c"tar.gz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static H2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"h2".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"H2 database".as_ptr(),
    extensions: [c"h2.db".as_ptr(), c"mv.db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static H5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"h5".as_ptr(),
    mime_type: c"application/x-hdf5".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Hierarchical Data Format v5".as_ptr(),
    extensions: [c"h5".as_ptr(), c"hdf5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HA_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ha_archive".as_ptr(),
    mime_type: c"application/x-ha-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Harri Hirvola HA Compressed Archive (.ha)".as_ptr(),
    extensions: [c"ha".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HAIKU_VECTOR_ICON_FORMAT_HVIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"haiku_vector_icon_format_hvif".as_ptr(),
    mime_type: c"application/x-haiku-vector-icon-format-hvif".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Haiku Vector Icon Format (.hvif, PRONOM fmt/1791)".as_ptr(),
    extensions: [c"hvif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HALO_BLF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"halo_blf".as_ptr(),
    mime_type: c"application/x-halo-blf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Bungie Halo BLF Container / Forge Variant (.blf / .map, 2007) Bungie".as_ptr(),
    extensions: [c"blf".as_ptr(), c"map".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HALPST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"halpst".as_ptr(),
    mime_type: c"audio/x-halpst".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"HAL Laboratory GameCube HALPST Audio Stream (.hps)".as_ptr(),
    extensions: [c"hps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HANDLEBARS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"handlebars".as_ptr(),
    mime_type: c"text/x-handlebars-template".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Handlebars source".as_ptr(),
    extensions: [c"hbs".as_ptr(), c"handlebars".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HANMAC_WORD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hanmac_word".as_ptr(),
    mime_type: c"application/x-hanmac-word".as_ptr(),
    group: c"document".as_ptr(),
    description: c"HanMac Word-J / Word-K Macintosh Japanese/Korean Document (.hj / .hk, YLYS / HYLS)".as_ptr(),
    extensions: [c"hj".as_ptr(), c"hk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HARVARD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"harvard".as_ptr(),
    mime_type: c"application/x-harvard-graphics".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Software Publishing Harvard Graphics Presentation / Chart (.ch3 / .sh3)".as_ptr(),
    extensions: [c"ch3".as_ptr(), c"sh3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HASKELL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"haskell".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Haskell source".as_ptr(),
    extensions: [c"hs".as_ptr(), c"lhs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HAVOK_HKX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"havok_hkx".as_ptr(),
    mime_type: c"application/x-havok-hkx".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Havok Physics / Animation Binary Tagfile & Packfile (.hkx / .hkt, 2004) Havok".as_ptr(),
    extensions: [c"hkx".as_ptr(), c"hkt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HCL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hcl".as_ptr(),
    mime_type: c"text/x-hcl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"HashiCorp configuration language".as_ptr(),
    extensions: [c"hcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HDF4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hdf4".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"HDF4".as_ptr(),
    extensions: [c"h4".as_ptr(), c"hdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HDIFFPATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hdiffpatch".as_ptr(),
    mime_type: c"application/x-hdiffpatch".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"HouSisong HDiffPatch Binary Differential Patch".as_ptr(),
    extensions: [c"hdiff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HDR_RGBE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hdr_rgbe".as_ptr(),
    mime_type: c"image/vnd.radiance".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Radiance RGBE HDR image (.hdr/.pic)".as_ptr(),
    extensions: [c"hdr".as_ptr(), c"pic".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HEIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"heif".as_ptr(),
    mime_type: c"image/heic".as_ptr(),
    group: c"image".as_ptr(),
    description: c"High Efficiency Image File".as_ptr(),
    extensions: [c"heif".as_ptr(), c"heifs".as_ptr(), c"heic".as_ptr(), c"heics".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HELIOS_SAVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"helios_save".as_ptr(),
    mime_type: c"application/x-helios-save".as_ptr(),
    group: c"application".as_ptr(),
    description: c"HELIOS Save State Container (.save)".as_ptr(),
    extensions: [c"save".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HELPPC_INDEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"helppc_index".as_ptr(),
    mime_type: c"application/x-helppc-index".as_ptr(),
    group: c"database".as_ptr(),
    description: c"David Jurgens HelpPC Quick Reference Index (.ndx, 1991)".as_ptr(),
    extensions: [c"ndx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HEMERA_HTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hemera_hta".as_ptr(),
    mime_type: c"image/x-hemera-hta".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Hemera Photo-Objects Thumbnails Archive (.hta)".as_ptr(),
    extensions: [c"hta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HERAD_ADLIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"herad_adlib".as_ptr(),
    mime_type: c"audio/x-herad-adlib".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Herad / Cryo Interactive AdLib Music (.agd / .sdb / .sqx)".as_ptr(),
    extensions: [c"agd".as_ptr(), c"sdb".as_ptr(), c"sqx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HERCULES_DASD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hercules_dasd".as_ptr(),
    mime_type: c"application/x-hercules-dasd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Hercules IBM System/370 & z/Architecture CKD / FBA DASD Image (CKD_P370 / CKD_C370)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HERMES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hermes".as_ptr(),
    mime_type: c"application/x-hermes-bytecode".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"React Native Hermes JS bytecode (.hbc)".as_ptr(),
    extensions: [c"hbc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hes".as_ptr(),
    mime_type: c"audio/x-hes".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"PC Engine / TurboGrafx-16 Hudson Entertainment System Sound Format (.hes)".as_ptr(),
    extensions: [c"hes".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HFE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hfe".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"HxC Floppy Emulator disk image (.hfe)".as_ptr(),
    extensions: [c"hfe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hfs".as_ptr(),
    mime_type: c"application/x-hfs".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"hfs".as_ptr(),
    extensions: [c"hfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HFST_OL_TRANSDUCER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hfst_ol_transducer".as_ptr(),
    mime_type: c"application/x-hfst-ol-transducer".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Helsinki Finite-State Transducer Optimized Lookup (.hfstol)".as_ptr(),
    extensions: [c"hfstol".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HHD_LPACK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hhd_lpack".as_ptr(),
    mime_type: c"application/x-hhd-lpack".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"HHD Software Localization Pack (.lpack)".as_ptr(),
    extensions: [c"lpack".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIEW_EXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hiew_ext".as_ptr(),
    mime_type: c"application/x-hiew-ext".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SEN Hiew External Information / Names / Save File".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIGHLOGIC_FCP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"highlogic_fcp".as_ptr(),
    mime_type: c"application/x-highlogic-fcp".as_ptr(),
    group: c"font".as_ptr(),
    description: c"High-Logic FontCreator Font Project (.fcp, 2013) High-Logic".as_ptr(),
    extensions: [c"fcp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIKVISION_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hikvision_firmware".as_ptr(),
    mime_type: c"application/x-hikvision-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Hikvision DVR/NVR/IPC Firmware Package (.dav, 2012) Hikvision".as_ptr(),
    extensions: [c"dav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIPC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hipc".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jochen Hippel Amiga/Atari ST COSO Compressed Module (.hipc / .soc)".as_ptr(),
    extensions: [c"hipc".as_ptr(), c"soc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIPPEL_HIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hippel_hip".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jochen Hippel Amiga 68000 / 7-Voice Module (.hip / .hip7)".as_ptr(),
    extensions: [c"hip".as_ptr(), c"hip7".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIS_MASTERS_NOISE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"his_masters_noise".as_ptr(),
    mime_type: c"audio/x-his-masters-noise".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Mahoney & Kaktus His Master's Noise Amiga Module (.mod)".as_ptr(),
    extensions: [c"mod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HITACHI_HRF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hitachi_hrf".as_ptr(),
    mime_type: c"image/x-hitachi-hrf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Hitachi CADCore / Tracer Raster Format (.hrf)".as_ptr(),
    extensions: [c"hrf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HIVELY_TRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hively_tracker".as_ptr(),
    mime_type: c"audio/x-hvl".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga / Cross-Platform HivelyTracker Chiptune Module (.hvl)".as_ptr(),
    extensions: [c"hvl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hlp".as_ptr(),
    mime_type: c"application/winhlp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows help".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HOLY_NOISE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"holy_noise".as_ptr(),
    mime_type: c"audio/x-thn".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"The Holy Noise Amiga Audio Module (.thn)".as_ptr(),
    extensions: [c"thn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HOMEWORLD_LIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"homeworld_lif".as_ptr(),
    mime_type: c"image/x-homeworld-lif".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Relic Entertainment Homeworld Library Image Format Texture (.lif)".as_ptr(),
    extensions: [c"lif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HOMEWORLD_MEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"homeworld_mex".as_ptr(),
    mime_type: c"model/x-homeworld-mex".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Relic Entertainment Homeworld Mesh Extension Data (.mex)".as_ptr(),
    extensions: [c"mex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HOOPS_HSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hoops_hsf".as_ptr(),
    mime_type: c"model/x-hoops-hsf".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Tech Soft 3D HOOPS Stream File / SolidWorks eDrawings (.hsf / .easm)".as_ptr(),
    extensions: [c"hsf".as_ptr(), c"easm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP2100_ABS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp2100_abs".as_ptr(),
    mime_type: c"application/x-hp2100-abs".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Hewlett-Packard HP 2100 / HP 1000 Absolute Binary Paper Tape (.hpb / .abs, 1968) Hewlett-Packard".as_ptr(),
    extensions: [c"hpb".as_ptr(), c"abs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP2100_REL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp2100_rel".as_ptr(),
    mime_type: c"application/x-hp2100-rel".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Hewlett-Packard HP 2100 / HP 1000 Relocatable Binary Paper Tape (.hpr / .rel, 1968) Hewlett-Packard".as_ptr(),
    extensions: [c"hpr".as_ptr(), c"rel".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP48_BIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp48_bin".as_ptr(),
    mime_type: c"application/x-hp48".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"HP 48 / HP 49 Calculator Binary Object (.hp / .grb)".as_ptr(),
    extensions: [c"hp".as_ptr(), c"grb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP_BDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp_bdl".as_ptr(),
    mime_type: c"application/x-hp-bdl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"HP FutureSmart Printer Firmware Bundle (.bdl)".as_ptr(),
    extensions: [c"bdl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP_IPKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp_ipkg".as_ptr(),
    mime_type: c"application/x-hp-ipkg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"HP LaserJet IPKG Firmware Package (.ipkg)".as_ptr(),
    extensions: [c"ipkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP_PALMTOP_EXM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp_palmtop_exm".as_ptr(),
    mime_type: c"application/x-hp-exm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"HP 95LX / 100LX / 200LX System Manager Executable (.exm)".as_ptr(),
    extensions: [c"exm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HP_PCL_SOFTFONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hp_pcl_softfont".as_ptr(),
    mime_type: c"font/x-hp-pcl".as_ptr(),
    group: c"font".as_ptr(),
    description: c"HP PCL / PaintJet Downloadable Soft Font (.sfp / .sfl / .pjf, 1985) Hewlett-Packard".as_ptr(),
    extensions: [c"sfp".as_ptr(), c"sfl".as_ptr(), c"pjf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HPDT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hpdt".as_ptr(),
    mime_type: c"application/x-highway-pursuit-hpdt".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Adam Dawson Highway Pursuit Game Data Archive (.h*d, 2003) Retrospec".as_ptr(),
    extensions: [c"h".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HPUX_SOM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hpux_som".as_ptr(),
    mime_type: c"application/x-hpux-som".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"HP-UX PA-RISC / M68k System Object Model (SOM) Executable & Object (.sl / .o, 1985) Hewlett-Packard".as_ptr(),
    extensions: [c"sl".as_ptr(), c"o".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HSC_TRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hsc_tracker".as_ptr(),
    mime_type: c"audio/x-hsc-tracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Hannes Seifert HSC-Tracker OPL2 Module (.hsc)".as_ptr(),
    extensions: [c"hsc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HSI_JPEG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hsi_jpeg".as_ptr(),
    mime_type: c"image/x-hsi-jpeg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Handmade Software Image Alchemy HSI JPEG (.jpg / .hsi)".as_ptr(),
    extensions: [c"jpg".as_ptr(), c"hsi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HSI_RAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hsi_raw".as_ptr(),
    mime_type: c"image/x-hsi-raw".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Handmade Software Image Alchemy HSI Raw Bitmap (.raw)".as_ptr(),
    extensions: [c"raw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hta".as_ptr(),
    mime_type: c"application/hta".as_ptr(),
    group: c"code".as_ptr(),
    description: c"HTML Application".as_ptr(),
    extensions: [c"hta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HTACCESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"htaccess".as_ptr(),
    mime_type: c"text/x-apache-conf".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Apache access configuration".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HTML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"html".as_ptr(),
    mime_type: c"text/html".as_ptr(),
    group: c"code".as_ptr(),
    description: c"HTML document".as_ptr(),
    extensions: [c"html".as_ptr(), c"htm".as_ptr(), c"xhtml".as_ptr(), c"xht".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static HUAWEI_UPDATE_APP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"huawei_update_app".as_ptr(),
    mime_type: c"application/x-huawei-update-app".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Huawei / HiSilicon Firmware Update Package (UPDATE.APP) Huawei".as_ptr(),
    extensions: [c"app".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hve".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"hve".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HWP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hwp".as_ptr(),
    mime_type: c"application/x-hwp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Hangul Word Processor".as_ptr(),
    extensions: [c"hwp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HXNF_TROUBLESHOOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hxnf_troubleshoot".as_ptr(),
    mime_type: c"application/x-hxnf-troubleshoot".as_ptr(),
    group: c"application".as_ptr(),
    description: c"HXNF Troubleshoot Diagnostic Dump (.troubleshoot)".as_ptr(),
    extensions: [c"troubleshoot".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HYPER_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hyper_archive".as_ptr(),
    mime_type: c"application/x-hyper-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Hyper Compressed Archive (.hyp)".as_ptr(),
    extensions: [c"hyp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HYPERPLANNING_HYP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hyperplanning_hyp".as_ptr(),
    mime_type: c"application/x-hyperplanning-hyp".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Index Education HYPERPLANNING Schedule Database (.hyp)".as_ptr(),
    extensions: [c"hyp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HYPERWRITER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hyperwriter".as_ptr(),
    mime_type: c"application/x-hyperwriter".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Ntergaid HyperWriter! Hypertext Document (.hw3 / .hw4)".as_ptr(),
    extensions: [c"hw3".as_ptr(), c"hw4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static HYTERA_FUDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"hytera_fudx".as_ptr(),
    mime_type: c"application/x-hytera-fudx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Hytera DMR Radio Firmware Upgrade Package (.fudx) Hytera Communications".as_ptr(),
    extensions: [c"fudx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_3740: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_3740".as_ptr(),
    mime_type: c"application/x-ibm-3740".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM 3740 / ECMA-58 Basic Exchange 8-Inch Floppy Image (.img / .dsk, 1973) IBM".as_ptr(),
    extensions: [c"img".as_ptr(), c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_DISPLAYWRITE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_displaywrite".as_ptr(),
    mime_type: c"application/x-ibm-displaywrite".as_ptr(),
    group: c"document".as_ptr(),
    description: c"IBM DisplayWrite 2/3/4/5 Document (.doc/.dw4, PRONOM x-fmt/288)".as_ptr(),
    extensions: [c"doc".as_ptr(), c"dw4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_MRNZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_mrnz".as_ptr(),
    mime_type: c"application/x-ibm-mrnz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM PC-DOS MRNZ Obfuscated Installer File (.co_ / .ex_ / .sy_)".as_ptr(),
    extensions: [c"co_".as_ptr(), c"ex_".as_ptr(), c"sy_".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_OS2_EXTENDED_ATTR_EA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_os2_extended_attr_ea".as_ptr(),
    mime_type: c"application/x-ibm-os2-extended-attr-ea".as_ptr(),
    group: c"application".as_ptr(),
    description: c"IBM OS/2 Extended Attributes Data File (EA DATA. SF, 1989) IBM".as_ptr(),
    extensions: [c"sf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_OS2_MKMSGF_MSG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_os2_mkmsgf_msg".as_ptr(),
    mime_type: c"application/x-ibm-os2-mkmsgf-msg".as_ptr(),
    group: c"document".as_ptr(),
    description: c"IBM OS/2 & PC DOS MKMSGF Compiled Message Catalog (.msg, 1987) IBM".as_ptr(),
    extensions: [c"msg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_SAVEDSKF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_savedskf".as_ptr(),
    mime_type: c"application/x-ibm-savedskf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM OS/2 SaveDskF / LoadDskF Floppy Disk Image (.dsk / .1dk / .2dk)".as_ptr(),
    extensions: [c"dsk".as_ptr(), c"1dk".as_ptr(), c"2dk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_SAVF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_savf".as_ptr(),
    mime_type: c"application/x-ibm-savf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM i / OS/400 Save File (SAVF)".as_ptr(),
    extensions: [c"savf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_STAP_CTL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_stap_ctl".as_ptr(),
    mime_type: c"application/x-ibm-stap-ctl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"IBM Guardium Windows S-TAP Circular Trace Log (.ctl)".as_ptr(),
    extensions: [c"ctl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBM_TERSE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibm_terse".as_ptr(),
    mime_type: c"application/x-ibm-terse".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM z/OS AMATERSE / TRSMAIN Tersed Data Set (PACK/SPACK)".as_ptr(),
    extensions: [c"terse".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBMPC_OVL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibmpc_ovl".as_ptr(),
    mime_type: c"application/x-ibmpc-ovl".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"IBM PC Compiler Runtime Overlay (.ovl)".as_ptr(),
    extensions: [c"ovl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IBUS_DICT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ibus_dict".as_ptr(),
    mime_type: c"application/x-ibus-dict".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Red Hat IBus GVariant Emoji Dictionary (.dict)".as_ptr(),
    extensions: [c"dict".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ICC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"icc".as_ptr(),
    mime_type: c"application/vnd.iccprofile".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"ICC profile".as_ptr(),
    extensions: [c"icc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ICNS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"icns".as_ptr(),
    mime_type: c"image/x-icns".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Mac OS X icon".as_ptr(),
    extensions: [c"icns".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ICO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ico".as_ptr(),
    mime_type: c"image/vnd.microsoft.icon".as_ptr(),
    group: c"image".as_ptr(),
    description: c"MS Windows icon resource".as_ptr(),
    extensions: [c"ico".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ICS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ics".as_ptr(),
    mime_type: c"text/calendar".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Internet Calendaring and Scheduling".as_ptr(),
    extensions: [c"ics".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ICY_TOWER_ITR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"icy_tower_itr".as_ptr(),
    mime_type: c"application/x-icy-tower-itr".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Free Lunch Design Icy Tower Replay Recording (ITR140, .itr)".as_ptr(),
    extensions: [c"itr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ID_ROQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"id_roq".as_ptr(),
    mime_type: c"video/x-roq".as_ptr(),
    group: c"video".as_ptr(),
    description: c"id Software Quake III / Doom 3 RoQ Video (.roq)".as_ptr(),
    extensions: [c"roq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ID_TED5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"id_ted5".as_ptr(),
    mime_type: c"application/x-id-ted5".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"id Software TED5 Tile Editor Game Maps Header (.wl1 / .wl6 / .map)".as_ptr(),
    extensions: [c"wl1".as_ptr(), c"wl6".as_ptr(), c"map".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IDATIL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"idatil".as_ptr(),
    mime_type: c"application/x-ida-til".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Hex-Rays IDA Pro Type Information Library (.til)".as_ptr(),
    extensions: [c"til".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"idb".as_ptr(),
    mime_type: c"application/x-ida-database".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Hex-Rays IDA Pro Disassembly Database (.idb / .i64)".as_ptr(),
    extensions: [c"idb".as_ptr(), c"i64".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IDR_KB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"idr_kb".as_ptr(),
    mime_type: c"application/x-idr-kb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Interactive Delphi Reconstructor Knowledge Base (.kb, 2009) crypto2011".as_ptr(),
    extensions: [c"kb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IDSP_SPR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"idsp_spr".as_ptr(),
    mime_type: c"application/x-idsp-spr".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"id Software Quake / Valve Half-Life IDSP 2D/3D Sprite (.spr, 1996) id Software / Valve".as_ptr(),
    extensions: [c"spr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IEBCOPY_PDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iebcopy_pds".as_ptr(),
    mime_type: c"application/x-ibm-iebcopy".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM z/OS IEBCOPY Unloaded Partitioned Data Set (COPYR1/COPYR2)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IFD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ifd".as_ptr(),
    mime_type: c"application/x-intel-ifd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel SPI Flash Descriptor (IFD)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IGES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iges".as_ptr(),
    mime_type: c"model/iges".as_ptr(),
    group: c"image".as_ptr(),
    description: c"IGES CAD exchange".as_ptr(),
    extensions: [c"iges".as_ptr(), c"igs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static IGNOREFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ignorefile".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Ignorefile".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static IGO_POI_KNS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"igo_poi_kns".as_ptr(),
    mime_type: c"application/x-igo-poi-kns".as_ptr(),
    group: c"database".as_ptr(),
    description: c"NNG iGO Navigation KML POI Search Index (.kns)".as_ptr(),
    extensions: [c"kns".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IL2CPP_METADATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"il2cpp_metadata".as_ptr(),
    mime_type: c"application/x-il2cpp-metadata".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Unity Engine IL2CPP Global Metadata Binary (global-metadata.dat)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ILBM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ilbm".as_ptr(),
    mime_type: c"image/x-ilbm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Amiga IFF ILBM image".as_ptr(),
    extensions: [c"iff".as_ptr(), c"ilbm".as_ptr(), c"lbm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ILLUSION_I3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"illusion_i3d".as_ptr(),
    mime_type: c"model/x-illusion-i3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Illusion Softworks LS3D Engine 3D Model (.i3d, 2002) Illusion Softworks".as_ptr(),
    extensions: [c"i3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMAGES_MUSIC_SYSTEM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"images_music_system".as_ptr(),
    mime_type: c"audio/x-images-music-system".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Images Design Images Music System / Unicorn Tracker Amiga Module (.ims)".as_ptr(),
    extensions: [c"ims".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMAGO_IMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"imago_imf".as_ptr(),
    mime_type: c"audio/x-imf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Lutz Roeder Imago Orpheus Tracker Module (.imf)".as_ptr(),
    extensions: [c"imf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"imd".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Dave Dunfield ImageDisk image (.imd)".as_ptr(),
    extensions: [c"imd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"img".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"img".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMP_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"imp_archive".as_ptr(),
    mime_type: c"application/x-imp-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Technelysium IMP Compressed Archive (.imp)".as_ptr(),
    extensions: [c"imp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMPLODER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"imploder".as_ptr(),
    mime_type: c"application/x-imploder".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Amiga File Imploder Compressed File (IMP!)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMUSE_MCMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"imuse_mcmp".as_ptr(),
    mime_type: c"audio/x-imuse-mcmp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"LucasArts iMUSE MCMP Compressed Audio (.imc)".as_ptr(),
    extensions: [c"imc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IMX_SDMA_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"imx_sdma_fw".as_ptr(),
    mime_type: c"application/x-imx-sdma-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NXP i.MX Smart Direct Memory Access Controller Firmware (SDMA)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IN_STEREO_IS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"in_stereo_is".as_ptr(),
    mime_type: c"audio/x-in-stereo".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Hans Wessels In Stereo! 1.0/2.0 Amiga Synthesizer Module (ISM!V1.2 / IS20DF10)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INDD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"indd".as_ptr(),
    mime_type: c"application/x-indesign".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Adobe InDesign Document / Book / Library (.indd / .indb / .indl)".as_ptr(),
    extensions: [c"indd".as_ptr(), c"indb".as_ptr(), c"indl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INFOCRYPT_ICSL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"infocrypt_icsl".as_ptr(),
    mime_type: c"application/x-infocrypt-icsl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Infocrypt ICS Encrypted Audit Log (.icsl)".as_ptr(),
    extensions: [c"icsl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INFOFILE_FLR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"infofile_flr".as_ptr(),
    mime_type: c"application/x-infofile-flr".as_ptr(),
    group: c"document".as_ptr(),
    description: c"InfoFile FLR Database (.flr, PRONOM fmt/1785)".as_ptr(),
    extensions: [c"flr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INFOGRAMES_INS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"infogrames_ins".as_ptr(),
    mime_type: c"audio/x-infogrames-ins".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Infogrames / Rob Hubbard II Amiga Sample Bank (.ins)".as_ptr(),
    extensions: [c"ins".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INFOTRIEVE_TPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"infotrieve_tpx".as_ptr(),
    mime_type: c"application/x-infotrieve-tpx".as_ptr(),
    group: c"database".as_ptr(),
    description: c"INFOTrieve Technologies TPX Record (.tpx, INFOTrieve Technologies)".as_ptr(),
    extensions: [c"tpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ini".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"INI configuration file".as_ptr(),
    extensions: [c"ini".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static INMAGIC_DBTEXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"inmagic_dbtext".as_ptr(),
    mime_type: c"application/x-inmagic-dbtextworks".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Inmagic DB/TextWorks Database File (.dbs / .btx / .tba / .ixl)".as_ptr(),
    extensions: [c"dbs".as_ptr(), c"btx".as_ptr(), c"tba".as_ptr(), c"ixl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INNO_SETUP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"inno_setup".as_ptr(),
    mime_type: c"application/x-innosetup".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Jordan Russell Inno Setup Installer Data / Log / Messages (.dat / .msg / .bin)".as_ptr(),
    extensions: [c"dat".as_ptr(), c"msg".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INNODB_TABLESPACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"innodb_tablespace".as_ptr(),
    mime_type: c"application/x-innodb-tablespace".as_ptr(),
    group: c"database".as_ptr(),
    description: c"MySQL / MariaDB InnoDB Tablespace (.ibd / ibdata1)".as_ptr(),
    extensions: [c"ibd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INSTALLER_VISE_XYZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"installer_vise_xyz".as_ptr(),
    mime_type: c"application/x-installer-vise-xyz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MindVision Installer VISE Metadata Container (ESIVCSIM, miscdata.xyz)".as_ptr(),
    extensions: [c"xyz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INSTALLSHIELD_INS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"installshield_ins".as_ptr(),
    mime_type: c"application/x-installshield-ins".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"InstallShield Compiled Script (.ins)".as_ptr(),
    extensions: [c"ins".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INSTALLSHIELD_INX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"installshield_inx".as_ptr(),
    mime_type: c"application/x-installshield-inx".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"InstallShield Compiled Rules Script (.inx / .ins)".as_ptr(),
    extensions: [c"inx".as_ptr(), c"ins".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INSTALLSHIELD_ISU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"installshield_isu".as_ptr(),
    mime_type: c"application/x-installshield-isu".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"InstallShield Uninstall Script / Log (.isu)".as_ptr(),
    extensions: [c"isu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INSTALLSHIELD_PKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"installshield_pkg".as_ptr(),
    mime_type: c"application/x-installshield-pkg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"InstallShield Package Directory (setup.pkg)".as_ptr(),
    extensions: [c"pkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INSTAR_BNEG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"instar_bneg".as_ptr(),
    mime_type: c"application/x-instar-bneg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"INSTAR IP Camera BNEG Firmware Container (.pkg)".as_ptr(),
    extensions: [c"pkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTEL_CPD_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intel_cpd_firmware".as_ptr(),
    mime_type: c"application/x-intel-cpd-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel Code Partition Directory Firmware Container ($CPD)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTEL_DVI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intel_dvi".as_ptr(),
    mime_type: c"video/x-intel-dvi".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Intel / IBM DVI ActionMedia AVSS Multimedia File (.dvi / .avs)".as_ptr(),
    extensions: [c"dvi".as_ptr(), c"avs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTEL_ESIF_DV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intel_esif_dv".as_ptr(),
    mime_type: c"application/x-intel-esif-dv".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel Dynamic Tuning Technology (DPTF) ESIF DataVault (.dv)".as_ptr(),
    extensions: [c"dv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTEL_ISCU_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intel_iscu_firmware".as_ptr(),
    mime_type: c"application/x-intel-iscu-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel C600 Storage Controller Unit Cable Management OEM Firmware (ISCUOEMB)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTEL_IVSC_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intel_ivsc_firmware".as_ptr(),
    mime_type: c"application/x-intel-ivsc-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel Vision Sensing Controller Firmware (SCVF / SFWI)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTEL_ME_FPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intel_me_fpt".as_ptr(),
    mime_type: c"application/x-intel-me-fpt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel Management Engine Flash Partition Table ($FPT, .bin) Intel".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTELHEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intelhex".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Intel HEX".as_ptr(),
    extensions: [c"hex".as_ptr(), c"ihex".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static INTELLIFONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"intellifont".as_ptr(),
    mime_type: c"application/x-font-intellifont".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Agfa Compugraphic Intellifont Scalable Font (.lib / .type)".as_ptr(),
    extensions: [c"lib".as_ptr(), c"type".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTERLEAVED_ADX_AUDIO_AIX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"interleaved_adx_audio_aix".as_ptr(),
    mime_type: c"application/x-interleaved-adx-audio-aix".as_ptr(),
    group: c"document".as_ptr(),
    description: c"CRI Middleware Interleaved ADX Audio (AIX) (.aix, PRONOM fmt/841)".as_ptr(),
    extensions: [c"aix".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static INTERNETSHORTCUT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"internetshortcut".as_ptr(),
    mime_type: c"application/x-mswinurl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows Internet shortcut".as_ptr(),
    extensions: [c"url".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static INTERPLAY_ACM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"interplay_acm".as_ptr(),
    mime_type: c"audio/x-interplay-acm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Interplay Entertainment ACM Compressed Audio (.acm)".as_ptr(),
    extensions: [c"acm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ION_PML_DGM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ion_pml_dgm".as_ptr(),
    mime_type: c"application/x-ion-pml-dgm".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Power Measurement / Schneider Electric ION Enterprise PML User Diagram (.dgm) Power Measurement".as_ptr(),
    extensions: [c"dgm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IOSAPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iosapp".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"iosapp".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IPYNB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ipynb".as_ptr(),
    mime_type: c"application/json".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Jupyter notebook".as_ptr(),
    extensions: [c"ipynb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static IRLOG1_LOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"irlog1_log".as_ptr(),
    mime_type: c"application/x-irlog1-log".as_ptr(),
    group: c"application".as_ptr(),
    description: c"IRLOG1 ECDSA-Signed Incident Response Log (.irlog)".as_ptr(),
    extensions: [c"irlog".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IS3_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"is3_archive".as_ptr(),
    mime_type: c"application/x-installshield".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"InstallShield v3 Compressed Installer Archive (_INST32I.EX_ / .EX_)".as_ptr(),
    extensions: [c"ex_".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IS_IBT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"is_ibt".as_ptr(),
    mime_type: c"application/x-installshield-compress-szdd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"InstallShield IBT Multi-Member SZDD Archive (setup.ibt)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ISCAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iscab".as_ptr(),
    mime_type: c"application/x-installshield".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"InstallShield Cabinet archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ISI_FRAC_CAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"isi_frac_car".as_ptr(),
    mime_type: c"application/x-isi-frac-car".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Image Space Incorporated FRAC Encrypted Vehicle Archive (.car, 2005) ISI".as_ptr(),
    extensions: [c"car".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ISI_GMOTOR2_MAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"isi_gmotor2_mas".as_ptr(),
    mime_type: c"application/x-isi-gmotor2-mas".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ISI gMotor 2 MAS Game Archive (.mas)".as_ptr(),
    extensions: [c"mas".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ISO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iso".as_ptr(),
    mime_type: c"application/x-iso9660-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ISO 9660 CD-ROM filesystem data".as_ptr(),
    extensions: [c"iso".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ISWLOG_SWL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iswlog_swl".as_ptr(),
    mime_type: c"application/x-iswlog-swl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ISWLOG Encrypted Diagnostic Trace (.swl)".as_ptr(),
    extensions: [c"swl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ISZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"isz".as_ptr(),
    mime_type: c"application/x-installshield".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"InstallShield Z / TSComp compressed archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"it".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Impulse Tracker Music Module (.it)".as_ptr(),
    extensions: [c"it".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IVF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ivf".as_ptr(),
    mime_type: c"video/x-ivf".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Duck TrueMotion / On2 / AOMedia IVF Video Container (.ivf)".as_ptr(),
    extensions: [c"ivf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IW4X_HEAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iw4x_heat".as_ptr(),
    mime_type: c"application/x-iw4x-heat".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IW4x Call of Duty: Modern Warfare 2 Multiplayer Heatmap (.iw4heat)".as_ptr(),
    extensions: [c"iw4heat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IWLWIFI_UCODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"iwlwifi_ucode".as_ptr(),
    mime_type: c"application/x-intel-iwlwifi-ucode".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Intel PRO/Wireless & Wi-Fi 6/7 Microcode Image (.ucode, IWL\\\\n)".as_ptr(),
    extensions: [c"ucode".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IX_INDEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ix_index".as_ptr(),
    mime_type: c"application/x-ix-index".as_ptr(),
    group: c"database".as_ptr(),
    description: c"IX Binary Database Index (.ix)".as_ptr(),
    extensions: [c"ix".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static IXALANCE_IXS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ixalance_ixs".as_ptr(),
    mime_type: c"audio/x-ixs".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Shortcut Ixalance Impulse Tracker Synth Module (.ixs)".as_ptr(),
    extensions: [c"ixs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static J3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"j3d".as_ptr(),
    mime_type: c"model/x-nintendo-j3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Nintendo GameCube / Wii JSystem 3D Model / Animation (.bmd / .bdl / .bck / .btk / .brk)".as_ptr(),
    extensions: [c"bmd".as_ptr(), c"bdl".as_ptr(), c"bck".as_ptr(), c"btk".as_ptr(), c"brk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JACKHAMMER_JMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jackhammer_jmf".as_ptr(),
    mime_type: c"application/x-jackhammer-jmf".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"J.A.C.K. / Jackhammer Binary Map (.jmf, 2006) Jackhammer team / Chain Studios".as_ptr(),
    extensions: [c"jmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAM_PACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jam_packer".as_ptr(),
    mime_type: c"application/x-atari-jam-packer".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST JAM Packer Compressed Data (LSD! / LZH! / LZW!)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAMCRACKER_JAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jamcracker_jam".as_ptr(),
    mime_type: c"audio/x-jamcracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"M. Gemmel JamCracker Pro Amiga Tracker Module (.jam, BeEp)".as_ptr(),
    extensions: [c"jam".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jar".as_ptr(),
    mime_type: c"application/java-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Java archive data (JAR)".as_ptr(),
    extensions: [c"jar".as_ptr(), c"klib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JASON_PAGE_OLD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jason_page_old".as_ptr(),
    mime_type: c"audio/x-jason-page-old".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jason Page Old Amiga Music Module (.jpo)".as_ptr(),
    extensions: [c"jpo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAVA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"java".as_ptr(),
    mime_type: c"text/x-java".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Java source".as_ptr(),
    extensions: [c"java".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JAVA_WEBSTART_IDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"java_webstart_idx".as_ptr(),
    mime_type: c"application/x-java-webstart-idx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Oracle Java Web Start Deployment Cache Index (.idx)".as_ptr(),
    extensions: [c"idx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAVABYTECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"javabytecode".as_ptr(),
    mime_type: c"application/x-java-applet".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Java compiled bytecode".as_ptr(),
    extensions: [c"class".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JAVASCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"javascript".as_ptr(),
    mime_type: c"application/javascript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"JavaScript source".as_ptr(),
    extensions: [c"js".as_ptr(), c"mjs".as_ptr(), c"cjs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JAYTRAX_JXS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jaytrax_jxs".as_ptr(),
    mime_type: c"audio/x-jaytrax".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Johannes Bjerregaard JayTrax Synth Module (.jxs)".as_ptr(),
    extensions: [c"jxs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JBIG2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jbig2".as_ptr(),
    mime_type: c"image/x-jbig2".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ITU-T T.88 / ISO/IEC 14492 JBIG2 Bi-Level Image (.jb2 / .jbig2)".as_ptr(),
    extensions: [c"jb2".as_ptr(), c"jbig2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JBOOT_SCH2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jboot_sch2".as_ptr(),
    mime_type: c"application/x-jboot-sch2".as_ptr(),
    group: c"application".as_ptr(),
    description: c"JBOOT SCH2 Second-Stage Kernel Header Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JEROEN_TEL_JT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jeroen_tel_jt".as_ptr(),
    mime_type: c"audio/x-jeroen-tel-jt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jeroen Tel / Maniacs of Noise Amiga Module (.jt)".as_ptr(),
    extensions: [c"jt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JETBRAINS_PHM_AT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jetbrains_phm_at".as_ptr(),
    mime_type: c"application/x-jetbrains-phm".as_ptr(),
    group: c"database".as_ptr(),
    description: c"JetBrains IntelliJ / Kotlin PersistentHashMap Value Storage (.values.at)".as_ptr(),
    extensions: [c"values".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JFFS2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jffs2".as_ptr(),
    mime_type: c"application/x-jffs2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Red Hat JFFS2 Flash Filesystem Image (.jffs2)".as_ptr(),
    extensions: [c"jffs2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JFX1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jfx1".as_ptr(),
    mime_type: c"application/x-jfx1".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"GMM Entertainment JFX1 / JPK1 Compressed Archive (.jpk)".as_ptr(),
    extensions: [c"jpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JGXP_PACKAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jgxp_package".as_ptr(),
    mime_type: c"application/x-jgxp-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"JGXP Game Resource Package (.gxp)".as_ptr(),
    extensions: [c"gxp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JIF99A: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jif99a".as_ptr(),
    mime_type: c"image/x-jif".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Jeff's Image Format (.jif, JIF99a)".as_ptr(),
    extensions: [c"jif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JINJA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jinja".as_ptr(),
    mime_type: c"text/x-jinja2-template".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Jinja template".as_ptr(),
    extensions: [c"jinja".as_ptr(), c"jinja2".as_ptr(), c"j2".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JKS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jks".as_ptr(),
    mime_type: c"application/x-java-keystore".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Java KeyStore (JKS/JCEKS)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jng".as_ptr(),
    mime_type: c"image/jng".as_ptr(),
    group: c"image".as_ptr(),
    description: c"JPEG network graphics".as_ptr(),
    extensions: [c"jng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JNLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jnlp".as_ptr(),
    mime_type: c"application/x-java-jnlp-file".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Java Network Launch Protocol".as_ptr(),
    extensions: [c"jnlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JODA_TZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"joda_tz".as_ptr(),
    mime_type: c"application/x-joda-zoneinfo".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Joda-Time Compiled Time Zone (DateTimeZone)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JP2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jp2".as_ptr(),
    mime_type: c"image/jpeg2000".as_ptr(),
    group: c"image".as_ptr(),
    description: c"jpeg2000".as_ptr(),
    extensions: [c"jp2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JP2_PGX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jp2_pgx".as_ptr(),
    mime_type: c"image/x-jp2-pgx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ISO/IEC 15444-4 JPEG 2000 PGX Verification Component Image (.pgx)".as_ptr(),
    extensions: [c"pgx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JPEG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jpeg".as_ptr(),
    mime_type: c"image/jpeg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"JPEG image data".as_ptr(),
    extensions: [c"jpg".as_ptr(), c"jpeg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JPHP_MODULE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jphp_module".as_ptr(),
    mime_type: c"application/x-jphp-module".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"JPHP (DevelNext) Compiled PHP-on-JVM Module (.phb)".as_ptr(),
    extensions: [c"phb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JSON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"json".as_ptr(),
    mime_type: c"application/json".as_ptr(),
    group: c"code".as_ptr(),
    description: c"JSON document".as_ptr(),
    extensions: [c"json".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JSONL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jsonl".as_ptr(),
    mime_type: c"application/json".as_ptr(),
    group: c"code".as_ptr(),
    description: c"JSONL document".as_ptr(),
    extensions: [c"jsonl".as_ptr(), c"jsonld".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JSOS_MD8: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jsos_md8".as_ptr(),
    mime_type: c"application/x-jsos-md8".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"JSOS SOS Standard Filesystem 1.x (.md8, MatchWare)".as_ptr(),
    extensions: [c"md8".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JUCE_PROP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"juce_prop".as_ptr(),
    mime_type: c"application/x-juce-prop".as_ptr(),
    group: c"application".as_ptr(),
    description: c"JUCE PropertySet Binary Configuration (.settings, 2004) Raw Material Software".as_ptr(),
    extensions: [c"settings".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JUCE_VC2_PRESET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"juce_vc2_preset".as_ptr(),
    mime_type: c"application/x-juce-vc2-preset".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"JUCE AudioProcessorValueTreeState Binary XML Preset (VC2!, 2015) Raw Material Software / ROLI".as_ptr(),
    extensions: [c"p21preset".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JULIA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"julia".as_ptr(),
    mime_type: c"text/x-julia".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Julia source".as_ptr(),
    extensions: [c"jl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static JULIA_JI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"julia_ji".as_ptr(),
    mime_type: c"application/x-julia-ji".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Julia Project Precompiled Module Image (.ji)".as_ptr(),
    extensions: [c"ji".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JUMBF_C2PA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jumbf_c2pa".as_ptr(),
    mime_type: c"application/x-c2pa-manifest-store".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ISO/IEC 19566-5 JUMBF / C2PA Content Credentials Manifest (.jumbf / .c2pa, 2019) JPEG / C2PA".as_ptr(),
    extensions: [c"jumbf".as_ptr(), c"c2pa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JVC_EVERIO_MOI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jvc_everio_moi".as_ptr(),
    mime_type: c"application/x-jvc-moi".as_ptr(),
    group: c"video".as_ptr(),
    description: c"JVC Everio Camcorder Media Object Information Sidecar (.moi)".as_ptr(),
    extensions: [c"moi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JWLAUNCH_P2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jwlaunch_p2".as_ptr(),
    mime_type: c"application/x-jwlaunch-p2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Java Web Start Standalone Launcher Bundle (.p2)".as_ptr(),
    extensions: [c"p2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JXL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jxl".as_ptr(),
    mime_type: c"image/jxl".as_ptr(),
    group: c"image".as_ptr(),
    description: c"JPEG XL".as_ptr(),
    extensions: [c"jxl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static JXS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"jxs".as_ptr(),
    mime_type: c"image/jxs".as_ptr(),
    group: c"image".as_ptr(),
    description: c"JPEG XS image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KAPEL_MDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kapel_mdf".as_ptr(),
    mime_type: c"application/x-kapel-mdf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Lumex Kapel Capillary Electrophoresis Measurement (.mdf)".as_ptr(),
    extensions: [c"mdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KASPERSKY_UPDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kaspersky_upds".as_ptr(),
    mime_type: c"application/x-kaspersky-upds".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Kaspersky Anti-Virus UPDS Gzip Signature Database (.kdc, 2006) Kaspersky Lab".as_ptr(),
    extensions: [c"kdc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KD_RED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kd_red".as_ptr(),
    mime_type: c"application/x-kd-red".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Knowledge Dynamics RED Compressed Installer File (.001 / .red)".as_ptr(),
    extensions: [c"red".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KDE_KWALLET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kde_kwallet".as_ptr(),
    mime_type: c"application/x-kwallet".as_ptr(),
    group: c"application".as_ptr(),
    description: c"KDE KWallet Credential Store (.kwl)".as_ptr(),
    extensions: [c"kwl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KEEPASS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"keepass".as_ptr(),
    mime_type: c"application/x-keepass2".as_ptr(),
    group: c"database".as_ptr(),
    description: c"KeePass Password Safe Encrypted Database (.kdb / .kdbx)".as_ptr(),
    extensions: [c"kdb".as_ptr(), c"kdbx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KENS_ADLIB_KSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kens_adlib_ksm".as_ptr(),
    mime_type: c"audio/x-kens-ksm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ken Silverman AdLib Music (.ksm)".as_ptr(),
    extensions: [c"ksm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KENS_DIGITAL_KDM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kens_digital_kdm".as_ptr(),
    mime_type: c"audio/x-kens-kdm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ken Silverman Digital Music (.kdm)".as_ptr(),
    extensions: [c"kdm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KERAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"keras".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"model".as_ptr(),
    description: c"Keras model archive".as_ptr(),
    extensions: [c"keras".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KERBEROS_KIRBI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kerberos_kirbi".as_ptr(),
    mime_type: c"application/x-kerberos-kirbi".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Kerberos v5 KRB-CRED Credential Ticket (.kirbi)".as_ptr(),
    extensions: [c"kirbi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KEYCHAIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"keychain".as_ptr(),
    mime_type: c"application/x-apple-keychain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple macOS Keychain Encrypted Credential Store (.keychain)".as_ptr(),
    extensions: [c"keychain".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KEYENCE_LJ_LJHP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"keyence_lj_ljhp".as_ptr(),
    mime_type: c"application/x-keyence-ljhp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Keyence LJ-X8000 / LJ-V7000 Series Laser Profiler Settings (.ljhp)".as_ptr(),
    extensions: [c"ljhp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KGB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kgb".as_ptr(),
    mime_type: c"application/x-kgb-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"KGB Archiver Compressed Archive (.kgb)".as_ptr(),
    extensions: [c"kgb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KILL_LA_KILL_DGKP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kill_la_kill_dgkp".as_ptr(),
    mime_type: c"application/x-kill-la-kill-dgkp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"APLUS / Arc System Works Kill la Kill: IF DGKP Asset Archive (.pac / .lyt)".as_ptr(),
    extensions: [c"pac".as_ptr(), c"lyt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KINDLE_DRMION: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kindle_drmion".as_ptr(),
    mime_type: c"application/vnd.amazon.kfx-drm".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Amazon Kindle KFX DRM-Protected Ion Voucher (.kfx / .azw8, \\\\xEADRMION\\\\xEE)".as_ptr(),
    extensions: [c"kfx".as_ptr(), c"azw8".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KINDLE_KFX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kindle_kfx".as_ptr(),
    mime_type: c"application/vnd.amazon.kfx".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Amazon Kindle Format 10 Container (.kfx, CONT)".as_ptr(),
    extensions: [c"kfx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KLEI_KTEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"klei_ktex".as_ptr(),
    mime_type: c"image/x-klei-ktex".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Klei Entertainment KTEX Game Texture (.tex, 2013) Klei Entertainment".as_ptr(),
    extensions: [c"tex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kml".as_ptr(),
    mime_type: c"application/vnd.google-earth.kml+xml".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"Keyhole Markup Language".as_ptr(),
    extensions: [c"kml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static KMZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kmz".as_ptr(),
    mime_type: c"application/vnd.google-earth.kmz".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"Keyhole Markup Language archive".as_ptr(),
    extensions: [c"kmz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOALA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"koala".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Commodore 64 Koala Painter image".as_ptr(),
    extensions: [c"koa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KODAK_DC20_PRESET_PST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kodak_dc20_preset_pst".as_ptr(),
    mime_type: c"application/x-kodak-dc20-preset-pst".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Eastman Kodak DC20 Digital Camera Preset (.pst, 1996) Kodak".as_ptr(),
    extensions: [c"pst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KODAK_PHOTO_CD_INFO_PCD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kodak_photo_cd_info_pcd".as_ptr(),
    mime_type: c"application/x-kodak-photo-cd-info-pcd".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Eastman Kodak Photo CD Disc Information (INFO.PCD, 1992) Kodak".as_ptr(),
    extensions: [c"pcd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KODI_XBT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kodi_xbt".as_ptr(),
    mime_type: c"application/x-kodi-xbt".as_ptr(),
    group: c"image".as_ptr(),
    description: c"XBMC / Kodi XBTF Texture Bundle (.xbt)".as_ptr(),
    extensions: [c"xbt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOEI_LS11_S11: MagikaTypeInfo = MagikaTypeInfo {
    label: c"koei_ls11_s11".as_ptr(),
    mime_type: c"application/x-koei-ls11".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"KOEI Romance of the Three Kingdoms XI LS11 Data Archive (.s11)".as_ptr(),
    extensions: [c"s11".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOEI_TECMO_G1T: MagikaTypeInfo = MagikaTypeInfo {
    label: c"koei_tecmo_g1t".as_ptr(),
    mime_type: c"image/x-koei-tecmo-g1t".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Koei Tecmo G1T Texture Archive (.g1t)".as_ptr(),
    extensions: [c"g1t".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOEI_TECMO_XWS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"koei_tecmo_xws".as_ptr(),
    mime_type: c"audio/x-koei-tecmo-xws".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Koei Tecmo XWS Wave Stream Audio (.xws)".as_ptr(),
    extensions: [c"xws".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOHAN_SND: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kohan_snd".as_ptr(),
    mime_type: c"application/x-kohan-snd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"TimeGate Studios Kohan KA Sound Archive (.snd, 2001) TimeGate Studios".as_ptr(),
    extensions: [c"snd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOMPAS_3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kompas_3d".as_ptr(),
    mime_type: c"application/x-kompas-3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ASCON KOMPAS-3D CAD Drawing / Fragment / Model (.cdw / .frw / .m3d)".as_ptr(),
    extensions: [c"cdw".as_ptr(), c"frw".as_ptr(), c"m3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KONTAKT_HSIN_NKI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kontakt_hsin_nki".as_ptr(),
    mime_type: c"application/x-kontakt-hsin-nki".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Native Instruments Kontakt Monolith Instrument (.nki)".as_ptr(),
    extensions: [c"nki".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KONTAKT_NCW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kontakt_ncw".as_ptr(),
    mime_type: c"audio/x-kontakt-ncw".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Native Instruments Kontakt Lossless Compressed Wave (.ncw)".as_ptr(),
    extensions: [c"ncw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KONTEXT_LAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kontext_lat".as_ptr(),
    mime_type: c"application/x-kontext-lat".as_ptr(),
    group: c"application".as_ptr(),
    description: c"KONTEXT DOS Word Processor Printer/Keyboard Driver (.lat)".as_ptr(),
    extensions: [c"lat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KOTLIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kotlin".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Kotlin source".as_ptr(),
    extensions: [c"kt".as_ptr(), c"kts".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static KRITA_TILE_DATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"krita_tile_data".as_ptr(),
    mime_type: c"application/x-krita-tile-data".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Krita Layer Tile Data (.kra layer, 2009) KDE / Krita Foundation".as_ptr(),
    extensions: [c"kra".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KROM_MORK_RES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"krom_mork_res".as_ptr(),
    mime_type: c"application/x-krom-mork-res".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"KROM / MORK 64-Bit Resource Table (.1 / .10d0)".as_ptr(),
    extensions: [c"10d0".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KRYOFLUX_RAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kryoflux_raw".as_ptr(),
    mime_type: c"application/x-kryoflux-raw".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Software Preservation Society KryoFlux Raw Flux Stream (.raw)".as_ptr(),
    extensions: [c"raw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KSP_MU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ksp_mu".as_ptr(),
    mime_type: c"model/x-ksp-mu".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Squad Kerbal Space Program Part 3D Model (.mu)".as_ptr(),
    extensions: [c"mu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kss".as_ptr(),
    mime_type: c"audio/x-kss".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX / Sega Master System KSS Chiptune Audio (KSCC / KSSX)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kt".as_ptr(),
    mime_type: c"audio/x-kt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"kometbomb Klystrack Chiptune Song (.kt)".as_ptr(),
    extensions: [c"kt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KUDO_CATALOG_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kudo_catalog_db".as_ptr(),
    mime_type: c"application/x-kudo-kdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Imspace Kudo Image Browser Catalog Database (.kdb, 1992) Imspace Systems".as_ptr(),
    extensions: [c"kdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KUDO_HASH_INDEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kudo_hash_index".as_ptr(),
    mime_type: c"application/x-kudo-hsh".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Imspace Kudo Image Browser Hash Index (.hsh, 1992) Imspace Systems".as_ptr(),
    extensions: [c"hsh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KUGOU_MUSIC_KGM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kugou_music_kgm".as_ptr(),
    mime_type: c"application/x-kugou-music-kgm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"KuGou Music Encrypted Audio Container (.kgm / .vpr)".as_ptr(),
    extensions: [c"kgm".as_ptr(), c"vpr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KVAG_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kvag_audio".as_ptr(),
    mime_type: c"audio/x-kvag".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Simon & Schuster Interactive KVAG ADPCM Audio (.vag)".as_ptr(),
    extensions: [c"vag".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KWAJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kwaj".as_ptr(),
    mime_type: c"application/x-ms-compress".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft KWAJ compressed member".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KWIKDRAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kwikdraw".as_ptr(),
    mime_type: c"image/x-kwikdraw".as_ptr(),
    group: c"image".as_ptr(),
    description: c"KwikDraw Object-Oriented Drawing (.kwk)".as_ptr(),
    extensions: [c"kwk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static KYOCERA_RAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"kyocera_raw".as_ptr(),
    mime_type: c"image/x-kyocera-raw".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Kyocera Contax N Digital RAW Image (.raw, 2002) Kyocera".as_ptr(),
    extensions: [c"raw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LA_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"la_audio".as_ptr(),
    mime_type: c"audio/x-la".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Michael Bevin La Lossless Audio Compressed Wave (.la)".as_ptr(),
    extensions: [c"la".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LABGRUPPEN_IPXPRESET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"labgruppen_ipxpreset".as_ptr(),
    mime_type: c"application/x-labgruppen-ipxpreset".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Lab.gruppen IPX Series Amplifier DSP Preset (.ipxpreset) Lab.gruppen".as_ptr(),
    extensions: [c"ipxpreset".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LARIAN_LSOF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"larian_lsof".as_ptr(),
    mime_type: c"application/x-larian-lsof".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Larian Studios LSOF Binary Resource (.lsf, 2014) Larian Studios".as_ptr(),
    extensions: [c"lsf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LARIAN_LSPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"larian_lspk".as_ptr(),
    mime_type: c"application/x-larian-lspk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Larian Studios Divinity / Baldur's Gate 3 LSPK Package (.pak / .lsv, 2014) Larian Studios".as_ptr(),
    extensions: [c"pak".as_ptr(), c"lsv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"las".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"LAS point cloud".as_ptr(),
    extensions: [c"las".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LATEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"latex".as_ptr(),
    mime_type: c"text/x-tex".as_ptr(),
    group: c"text".as_ptr(),
    description: c"LaTeX document".as_ptr(),
    extensions: [c"tex".as_ptr(), c"sty".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LATTICE_BITSTREAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lattice_bitstream".as_ptr(),
    mime_type: c"application/x-lattice-bitstream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Lattice iCE40 / ECP5 / MachXO / Nexus FPGA Bitstream (.bit / .bin) Lattice Semiconductor".as_ptr(),
    extensions: [c"bit".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LAXITY_PT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"laxity_pt".as_ptr(),
    mime_type: c"audio/x-laxity-pt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Laxity Amiga Powertracker Module (.pt)".as_ptr(),
    extensions: [c"pt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LAYAAIR_MESH_LM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"layaair_mesh_lm".as_ptr(),
    mime_type: c"model/x-layaair-lm".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"LayaAir 3D Engine Binary Mesh (.lm)".as_ptr(),
    extensions: [c"lm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LBR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lbr".as_ptr(),
    mime_type: c"application/x-cpm-lbr".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CP/M LU / LBR Library Archive (.lbr)".as_ptr(),
    extensions: [c"lbr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LCIO_SIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lcio_sio".as_ptr(),
    mime_type: c"application/x-lcio-sio".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Linear Collider I/O SIO Event Stream (.slcio)".as_ptr(),
    extensions: [c"slcio".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ldb".as_ptr(),
    mime_type: c"application/x-lcf-database".as_ptr(),
    group: c"database".as_ptr(),
    description: c"ASCII / Enterbrain RPG Maker 2000 / 2003 LCF Database / Map / Save (.ldb / .lmt / .lmu / .lsd)".as_ptr(),
    extensions: [c"ldb".as_ptr(), c"lmt".as_ptr(), c"lmu".as_ptr(), c"lsd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEADTOOLS_CMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leadtools_cmp".as_ptr(),
    mime_type: c"image/x-leadtools-cmp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"LEADTOOLS Compressed Image (.cmp, PRONOM fmt/1337)".as_ptr(),
    extensions: [c"cmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEGGLESS_LME: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leggless_lme".as_ptr(),
    mime_type: c"audio/x-lme".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Leggless / Silents UK Amiga Music Editor Module (.lme)".as_ptr(),
    extensions: [c"lme".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEGO_ALP_TUN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lego_alp_tun".as_ptr(),
    mime_type: c"audio/x-lego-alp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"High Voltage Software / LEGO Racers ALP Audio (.tun)".as_ptr(),
    extensions: [c"tun".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEICA_APSC_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leica_apsc_firmware".as_ptr(),
    mime_type: c"application/x-leica-apsc-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Leica Camera APS-C ASCII-Headered Firmware Update (.upd / .tuf / .lfu, 2009) Leica Camera".as_ptr(),
    extensions: [c"upd".as_ptr(), c"tuf".as_ptr(), c"lfu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEICA_DMR_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leica_dmr_firmware".as_ptr(),
    mime_type: c"application/x-leica-dmr-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Leica Digital Modul-R XOR-0x35 Firmware Image (.cim, 2005) Leica / Imacon".as_ptr(),
    extensions: [c"cim".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEICA_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leica_firmware".as_ptr(),
    mime_type: c"application/x-leica-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Leica Camera Maestro / Fujitsu Firmware Container (.fw / .s2, 2009) Leica Camera".as_ptr(),
    extensions: [c"fw".as_ptr(), c"s2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEICA_LENS_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leica_lens_db".as_ptr(),
    mime_type: c"application/x-leica-lens-db".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Leica Camera Lens Correction & Calibration Database (2010) Leica Camera".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LENOVO_LOGDAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lenovo_logdat".as_ptr(),
    mime_type: c"application/x-lenovo-logdat".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Lenovo Smart Engine XOR-Obfuscated Installer Log (.logdat)".as_ptr(),
    extensions: [c"logdat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEONARDO_IMAGE_FORMAT_LEO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leonardo_image_format_leo".as_ptr(),
    mime_type: c"application/x-leonardo-image-format-leo".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Leonardo Image Format (.leo, PRONOM fmt/1217)".as_ptr(),
    extensions: [c"leo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LEVELDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"leveldb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"LevelDB".as_ptr(),
    extensions: [c"ldb".as_ptr(), c"log".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LFM2_LFMAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lfm2_lfmat".as_ptr(),
    mime_type: c"application/x-lfm2-lfmat".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"LFM2 3D Material Definition (.lfmat)".as_ptr(),
    extensions: [c"lfmat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LHA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lha".as_ptr(),
    mime_type: c"application/x-lha".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LHarc archive".as_ptr(),
    extensions: [c"lha".as_ptr(), c"lzh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIBGDX_G3DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"libgdx_g3db".as_ptr(),
    mime_type: c"model/x-libgdx-g3db".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"libGDX UBJSON Binary 3D Model (.g3db)".as_ptr(),
    extensions: [c"g3db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIBLZG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"liblzg".as_ptr(),
    mime_type: c"application/x-lzg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Marcus Geelnard liblzg Compressed Stream (.lzg)".as_ptr(),
    extensions: [c"lzg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIGHTWAVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lightwave".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"LightWave object / scene".as_ptr(),
    extensions: [c"lwo".as_ptr(), c"lws".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LINEAGE2_UNREAL_PKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lineage2_unreal_pkg".as_ptr(),
    mime_type: c"application/x-lineage2-unreal-pkg".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"NCSoft Lineage II Encrypted Unreal Engine Package (Lineage2Ver, .usx / .ukx / .utx)".as_ptr(),
    extensions: [c"usx".as_ptr(), c"ukx".as_ptr(), c"utx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LINGEA_LLD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lingea_lld".as_ptr(),
    mime_type: c"application/x-lingea-lld".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Lingea Lexicon Bilingual Dictionary Database (.lld, 2002) Lingea s.r.o.".as_ptr(),
    extensions: [c"lld".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LINUX_KMOD_INDEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"linux_kmod_index".as_ptr(),
    mime_type: c"application/x-linux-kmod-index".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Linux kmod / module-init-tools Binary Module Index (.bin, 2009) Alan Jenkins".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LINUX_PERF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"linux_perf".as_ptr(),
    mime_type: c"application/x-linux-perf-data".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Linux Kernel perf.data Performance Profile (PERFILE2)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LINUX_PSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"linux_psf".as_ptr(),
    mime_type: c"application/x-font-linux-psf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Linux Console PC Screen Font (PSF1 / PSF2)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIONHEART_LION: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lionheart_lion".as_ptr(),
    mime_type: c"audio/x-lionheart".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Thalion Lionheart Amiga Music Module (.lion)".as_ptr(),
    extensions: [c"lion".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"liq".as_ptr(),
    mime_type: c"audio/x-liq".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nir Oren Liquid Tracker Module (.liq)".as_ptr(),
    extensions: [c"liq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LISP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lisp".as_ptr(),
    mime_type: c"text/x-lisp".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Lisp source".as_ptr(),
    extensions: [c"lisp".as_ptr(), c"lsp".as_ptr(), c"l".as_ptr(), c"cl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LITHTECH_REZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lithtech_rez".as_ptr(),
    mime_type: c"application/x-lithtech-rez".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Monolith LithTech Engine RezMgr Resource Archive (.rez, 1998) Monolith Productions".as_ptr(),
    extensions: [c"rez".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LITTLEFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"littlefs".as_ptr(),
    mime_type: c"application/x-littlefs-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ARM Mbed littlefs Microcontroller Flash Filesystem".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIVE2D_CMO3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"live2d_cmo3".as_ptr(),
    mime_type: c"application/x-live2d-cmo3".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Live2D Cubism 3+ Editor Model Project (.cmo3 / .can3, 2017) Live2D".as_ptr(),
    extensions: [c"cmo3".as_ptr(), c"can3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LIVECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"livecode".as_ptr(),
    mime_type: c"application/x-livecode-stack".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Runtime Revolution / LiveCode Stack (.rev / .livecode)".as_ptr(),
    extensions: [c"rev".as_ptr(), c"livecode".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LLVM_BITCODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"llvm_bitcode".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"LLVM bitcode".as_ptr(),
    extensions: [c"bc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LLVM_IR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"llvm_ir".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"LLVM textual IR".as_ptr(),
    extensions: [c"ll".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LLVM_PROFDATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"llvm_profdata".as_ptr(),
    mime_type: c"application/x-llvm-profdata".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"LLVM InstrProf Indexed / Raw Coverage Profile Data (.profdata / .profraw)".as_ptr(),
    extensions: [c"profdata".as_ptr(), c"profraw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LMDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lmdb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"LMDB".as_ptr(),
    extensions: [c"mdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lnk".as_ptr(),
    mime_type: c"application/x-ms-shortcut".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Windows shortcut".as_ptr(),
    extensions: [c"lnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LOB_MULTIPAK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lob_multipak".as_ptr(),
    mime_type: c"application/x-lob-multipak".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LOB's File Compressor / Multipak Amiga Data Stream (.lob)".as_ptr(),
    extensions: [c"lob".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LOCALE_STRNDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"locale_strndx".as_ptr(),
    mime_type: c"application/x-locale-strndx".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Sequential Offset/Length Localization String Index (.strndx)".as_ptr(),
    extensions: [c"strndx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LOCOSCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"locoscript".as_ptr(),
    mime_type: c"application/x-locoscript".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Locomotive Software LocoScript Document (.loc)".as_ptr(),
    extensions: [c"loc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LOGITECH_COMPRESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"logitech_compress".as_ptr(),
    mime_type: c"application/x-logitech-compress".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Logitech LGEXPAND Compressed Installer File (.ex_ / .dl_ / .cu_)".as_ptr(),
    extensions: [c"ex_".as_ptr(), c"dl_".as_ptr(), c"cu_".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LORICIEL_BNKS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"loriciel_bnks".as_ptr(),
    mime_type: c"audio/x-loriciel-bnks".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Loriciel Amiga Sample Bank (.bnk)".as_ptr(),
    extensions: [c"bnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LOTUS_123_INSTALL_DAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lotus_123_install_dat".as_ptr(),
    mime_type: c"application/x-lotus-123-install-dat".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Lotus 1-2-3 Release 3 Installation Driver Database (LOTUS.DAT, 1989) Lotus".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LOTUS_CMZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lotus_cmz".as_ptr(),
    mime_type: c"application/x-lotus-cmz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Lotus Ami Pro CMZ Compressed Archive (.cmz)".as_ptr(),
    extensions: [c"cmz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LPAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lpac".as_ptr(),
    mime_type: c"audio/x-lpac".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Tilman Liebchen LPAC Lossless Predictive Audio Compressor (.pac, 1999) TU Berlin".as_ptr(),
    extensions: [c"pac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LRZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lrz".as_ptr(),
    mime_type: c"application/x-lrzip".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"LRZip".as_ptr(),
    extensions: [c"lrz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LSSTAT_STAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lsstat_stat".as_ptr(),
    mime_type: c"application/x-lsstat-stat".as_ptr(),
    group: c"application".as_ptr(),
    description: c"LSSTAT License/System Status Record (.stat)".as_ptr(),
    extensions: [c"stat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lua".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Lua".as_ptr(),
    extensions: [c"lua".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static LUABYTECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"luabytecode".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Lua bytecode".as_ptr(),
    extensions: [c"luac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUCASARTS_GOB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lucasarts_gob".as_ptr(),
    mime_type: c"application/x-lucasarts-gob".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LucasArts Dark Forces GOB Archive (.gob)".as_ptr(),
    extensions: [c"gob".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUCASARTS_LAA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lucasarts_laa".as_ptr(),
    mime_type: c"audio/x-lucasarts-laa".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"LucasArts AdLib Audio Music (.laa)".as_ptr(),
    extensions: [c"laa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUCASARTS_SOU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lucasarts_sou".as_ptr(),
    mime_type: c"audio/x-lucasarts-sou".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"LucasArts iMUSE / Monster.SOU VCTL Voice Audio (.sou)".as_ptr(),
    extensions: [c"sou".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUCENE_SI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lucene_si".as_ptr(),
    mime_type: c"application/x-lucene-codec".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Apache Lucene Codec Segment Index File (.si / .cfe / .cfs)".as_ptr(),
    extensions: [c"si".as_ptr(), c"cfe".as_ptr(), c"cfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LURAWAVE_LWF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lurawave_lwf".as_ptr(),
    mime_type: c"image/x-lurawave".as_ptr(),
    group: c"image".as_ptr(),
    description: c"LuraTech LuraWave Wavelet Bitmap (.lwf)".as_ptr(),
    extensions: [c"lwf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LUXONIX_PURITY_PTH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"luxonix_purity_pth".as_ptr(),
    mime_type: c"application/x-luxonix-purity-pth".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Luxonix / Sonic Cat Purity Rompler Synthesizer Patch (.pth, 2005) Luxonix".as_ptr(),
    extensions: [c"pth".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LYTRO_LFP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lytro_lfp".as_ptr(),
    mime_type: c"application/x-lytro-lfp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Lytro Light Field Picture Container (.lfp / .lfr)".as_ptr(),
    extensions: [c"lfp".as_ptr(), c"lfr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lz".as_ptr(),
    mime_type: c"application/x-lzip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LZip".as_ptr(),
    extensions: [c"lz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZ4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lz4".as_ptr(),
    mime_type: c"application/x-lz4".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LZ4".as_ptr(),
    extensions: [c"lz4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzf".as_ptr(),
    mime_type: c"application/x-lzf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LZF compressed stream".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZFSE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzfse".as_ptr(),
    mime_type: c"application/x-lzfse".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple LZFSE / LZVN compressed stream".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZM_COMPRESSED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzm_compressed".as_ptr(),
    mime_type: c"application/x-lzm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LZM1 Chunked Compressed Stream (.lzm)".as_ptr(),
    extensions: [c"lzm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZOP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzop".as_ptr(),
    mime_type: c"application/x-lzop".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"lzop LZO compressed file (.lzo)".as_ptr(),
    extensions: [c"lzo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZP1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzp1".as_ptr(),
    mime_type: c"application/x-lzp1".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Charles Bloom LZP1 Compressed Stream (.lzp)".as_ptr(),
    extensions: [c"lzp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZSA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzsa".as_ptr(),
    mime_type: c"application/x-lzsa".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Emmanuel Marty LZSA Compressed Stream (.lzsa)".as_ptr(),
    extensions: [c"lzsa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static LZX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"lzx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"lzx".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static M3U: MagikaTypeInfo = MagikaTypeInfo {
    label: c"m3u".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"M3U playlist".as_ptr(),
    extensions: [c"m3u8".as_ptr(), c"m3u".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static M4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"m4".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"GNU Macro".as_ptr(),
    extensions: [c"m4".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MAC_WORD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mac_word".as_ptr(),
    mime_type: c"application/msword".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Word for Macintosh Classic Document (.mcw, v1-v5)".as_ptr(),
    extensions: [c"mcw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MACHO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"macho".as_ptr(),
    mime_type: c"application/x-mach-o".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Mach-O executable".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MACOS_FSEVENTS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"macos_fsevents".as_ptr(),
    mime_type: c"application/x-apple-fsevents".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple macOS FSEvents File System Event Log (1SLD / 2SLD / 3SLD)".as_ptr(),
    extensions: [c"fseventsd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MACOS_UTMPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"macos_utmpx".as_ptr(),
    mime_type: c"application/x-apple-utmpx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Apple macOS User Accounting Log (utmpx)".as_ptr(),
    extensions: [c"utmpx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MACRO_EXPRESS_MEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"macro_express_mex".as_ptr(),
    mime_type: c"application/x-macro-express-mex".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Insight Software Solutions Macro Express Macro File (.mex)".as_ptr(),
    extensions: [c"mex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MACWRITE2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"macwrite2".as_ptr(),
    mime_type: c"application/x-macwrite-ii".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Claris MacWrite II Document (.mw2)".as_ptr(),
    extensions: [c"mw2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"maff".as_ptr(),
    mime_type: c"application/x-maff".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"maff".as_ptr(),
    extensions: [c"maff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAGICAVOXEL_VOX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"magicavoxel_vox".as_ptr(),
    mime_type: c"model/x-magicavoxel".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ephtracy MagicaVoxel 3D Voxel Model (.vox, VOX )".as_ptr(),
    extensions: [c"vox".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAGNETIC_SCROLLS_GFX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"magnetic_scrolls_gfx".as_ptr(),
    mime_type: c"application/x-magnetic-scrolls-gfx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Magnetic Scrolls Picture Container (.gfx)".as_ptr(),
    extensions: [c"gfx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAGNETIC_SCROLLS_MAG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"magnetic_scrolls_mag".as_ptr(),
    mime_type: c"application/x-magnetic-scrolls".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Magnetic Scrolls Game Data (.mag)".as_ptr(),
    extensions: [c"mag".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAKEFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"makefile".as_ptr(),
    mime_type: c"text/x-makefile".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Makefile source".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MALCAT_CSIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"malcat_csig".as_ptr(),
    mime_type: c"application/x-malcat-csig".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Malcat Constant & Kesakode Signature Database (.csig)".as_ptr(),
    extensions: [c"csig".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MANDELBULB3D_M3P: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mandelbulb3d_m3p".as_ptr(),
    mime_type: c"application/x-mandelbulb3d-m3p".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Mandelbulb 3D Fractal Parameter Preset (.m3p)".as_ptr(),
    extensions: [c"m3p".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MANUSCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"manuscript".as_ptr(),
    mime_type: c"application/x-lotus-manuscript".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Lotus Manuscript Technical Document (.doc)".as_ptr(),
    extensions: [c"doc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAPINFO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mapinfo".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MapInfo dataset".as_ptr(),
    extensions: [c"map".as_ptr(), c"mid".as_ptr(), c"mif".as_ptr(), c"tab".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAPSFORGE_BINARY_VECTOR_MAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mapsforge_binary_vector_map".as_ptr(),
    mime_type: c"application/x-mapsforge-map".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Mapsforge Binary Vector Map (.map, PRONOM fmt/915)".as_ptr(),
    extensions: [c"map".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAR_ARCHIVE_MAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mar_archive_mac".as_ptr(),
    mime_type: c"application/x-mar-archive-mac".as_ptr(),
    group: c"document".as_ptr(),
    description: c"ListingPro / MAR Archive (.mac, PRONOM fmt/1483)".as_ptr(),
    extensions: [c"mac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MARINER_WRITE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mariner_write".as_ptr(),
    mime_type: c"application/x-mariner-write".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Mariner Write Classic Mac Document (.mwd)".as_ptr(),
    extensions: [c"mwd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MARK2_MK2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mark2_mk2".as_ptr(),
    mime_type: c"audio/x-mk2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Cachet Software Amiga Mark II Music System Module (.mk2)".as_ptr(),
    extensions: [c"mk2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MARK_COOKSEY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mark_cooksey".as_ptr(),
    mime_type: c"audio/x-mark-cooksey".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Mark Cooksey Amiga Music Module (.mc)".as_ptr(),
    extensions: [c"mc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MARKDOWN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"markdown".as_ptr(),
    mime_type: c"text/markdown".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Markdown document".as_ptr(),
    extensions: [c"md".as_ptr(), c"markdown".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MARMALADE_DZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"marmalade_dz".as_ptr(),
    mime_type: c"application/x-marmalade-dz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ideaworks3D Marmalade SDK Derbh Compressed Archive (.dz)".as_ptr(),
    extensions: [c"dz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MARVELL_CPT_UCODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"marvell_cpt_ucode".as_ptr(),
    mime_type: c"application/x-marvell-cpt-microcode".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Marvell OCTEON TX2 / CN10K Cryptographic Acceleration Engine Microcode (CPT01)".as_ptr(),
    extensions: [c"out".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mas".as_ptr(),
    mime_type: c"application/x-gmotor-mas".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ISI gMotor / rFactor MAS Game Data Archive (.mas / .gtl, GMOTORMAS10)".as_ptr(),
    extensions: [c"mas".as_ptr(), c"gtl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MASTER_TRACKS_MTS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"master_tracks_mts".as_ptr(),
    mime_type: c"audio/x-mts".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Passport Designs Master Tracks Pro Sequence (.mts)".as_ptr(),
    extensions: [c"mts".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MASTERCAM_V9_MC9: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mastercam_v9_mc9".as_ptr(),
    mime_type: c"application/x-mastercam-v9-mc9".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"CNC Software Mastercam v9 CAD/CAM Part (.mc9)".as_ptr(),
    extensions: [c"mc9".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MASTERTRACKER_MTR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mastertracker_mtr".as_ptr(),
    mime_type: c"audio/x-mtr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Kane Wood / Arkha Master Tracker Module (.mtr)".as_ptr(),
    extensions: [c"mtr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mat".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"MATLAB MAT data".as_ptr(),
    extensions: [c"mat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MATLAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"matlab".as_ptr(),
    mime_type: c"text/x-matlab".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Matlab Source".as_ptr(),
    extensions: [c"m".as_ptr(), c"matlab".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MATTER_OTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"matter_ota".as_ptr(),
    mime_type: c"application/x-matter-ota".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Connectivity Standards Alliance Matter OTA Software Update Image (.ota)".as_ptr(),
    extensions: [c"ota".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"max".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"3ds Max scene".as_ptr(),
    extensions: [c"max".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MAYA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"maya".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Maya scene".as_ptr(),
    extensions: [c"ma".as_ptr(), c"mb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MBMS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mbms".as_ptr(),
    mime_type: c"audio/x-msx-mbms".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX MoonBlaster for MoonSound OPL4 Music (.mbm)".as_ptr(),
    extensions: [c"mbm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MBTILES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mbtiles".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"MBTiles tile set".as_ptr(),
    extensions: [c"mbtiles".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MCAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mcap".as_ptr(),
    mime_type: c"application/vnd.mcap".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Foxglove Robotics MCAP Container (.mcap)".as_ptr(),
    extensions: [c"mcap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MCFT_CFT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mcft_cft".as_ptr(),
    mime_type: c"application/x-mcft-cft".as_ptr(),
    group: c"font".as_ptr(),
    description: c"MCFT Bitmap/Vector Font Resource (.cft)".as_ptr(),
    extensions: [c"cft".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MCMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mcmd".as_ptr(),
    mime_type: c"audio/x-mcmd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga MCMD Chiptune Module (.mcmd)".as_ptr(),
    extensions: [c"mcmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MCS_STIMULUS_STM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mcs_stimulus_stm".as_ptr(),
    mime_type: c"application/x-mcs-stimulus-stm".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Multi Channel Systems MC_Stimulus II Waveform Definition (.stm) Multi Channel Systems".as_ptr(),
    extensions: [c"stm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MCUBOOT_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mcuboot_img".as_ptr(),
    mime_type: c"application/x-mcuboot-image".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MCUboot / Apache Mynewt Secure Bootloader Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MD2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"md2".as_ptr(),
    mime_type: c"model/x-md2".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"id Software Quake II 3D Model (.md2)".as_ptr(),
    extensions: [c"md2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MDCD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mdcd".as_ptr(),
    mime_type: c"application/x-mdcd-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Dietrich / MDCD Compressed Archive (.mdcd)".as_ptr(),
    extensions: [c"mdcd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MDL_DIGITRAKKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mdl_digitrakker".as_ptr(),
    mime_type: c"audio/x-mdl".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Prodatron Digitrakker Module (.mdl)".as_ptr(),
    extensions: [c"mdl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mds".as_ptr(),
    mime_type: c"application/x-mds".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Alcohol 120% / DAEMON Tools Media Descriptor (.mds / .mdx)".as_ptr(),
    extensions: [c"mds".as_ptr(), c"mdx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MDXELM_G2P: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mdxelm_g2p".as_ptr(),
    mime_type: c"application/x-mdxelm-g2p".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"MDXELM DTPS 3D Part Element (.g2p)".as_ptr(),
    extensions: [c"g2p".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEDIATEK_BRLYT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mediatek_brlyt".as_ptr(),
    mime_type: c"application/x-mediatek-brlyt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MediaTek BootROM Layout Descriptor Header (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEDIATEK_BT_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mediatek_bt_fw".as_ptr(),
    mime_type: c"application/x-mediatek-bt-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MediaTek / ALPS Bluetooth Combo Controller Firmware Header (ALPS)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEDIATEK_IMAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mediatek_image".as_ptr(),
    mime_type: c"application/x-mediatek-image".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MediaTek Partition Image (0x58881688, .img / .bin) MediaTek".as_ptr(),
    extensions: [c"img".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEDIATEK_PRELOADER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mediatek_preloader".as_ptr(),
    mime_type: c"application/x-mediatek-preloader".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MediaTek EMMC_BOOT / SF_BOOT Preloader Image (.bin) MediaTek".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEDIATEK_SIGNED_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mediatek_signed_img".as_ptr(),
    mime_type: c"application/x-mediatek-signed-image".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MediaTek Signed Partition Image (BFBF / SSSS)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEDLEY_MSO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"medley_mso".as_ptr(),
    mime_type: c"audio/x-mso".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Medley Sound Amiga Module (.mso)".as_ptr(),
    extensions: [c"mso".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEGASTATION_MSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"megastation_msm".as_ptr(),
    mime_type: c"audio/x-megastation-midi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MegaStation Compressed MIDI Module (.msm)".as_ptr(),
    extensions: [c"msm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEGATRACKER_MGT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"megatracker_mgt".as_ptr(),
    mime_type: c"audio/x-mgt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Simplet / Fatal Design Atari Falcon MegaTracker Module (.mgt)".as_ptr(),
    extensions: [c"mgt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MEGAZEUX_WORLD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"megazeux_world".as_ptr(),
    mime_type: c"application/x-megazeux-world".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MegaZeux Game Creation System World (.mzx, 1994) Alexis Janson / DigitalMZX".as_ptr(),
    extensions: [c"mzx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MELEE_SLIPPI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"melee_slippi".as_ptr(),
    mime_type: c"application/x-melee-slippi".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Project Slippi Super Smash Bros. Melee Replay (.slp, 2018) Project Slippi".as_ptr(),
    extensions: [c"slp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MERCEDES_CFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mercedes_cff".as_ptr(),
    mime_type: c"application/x-mercedes-cff".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Mercedes-Benz Caesar CFF ECU Flash Firmware (.cff) Mercedes-Benz".as_ptr(),
    extensions: [c"cff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static METAKIT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"metakit".as_ptr(),
    mime_type: c"application/x-metakit".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Metakit v4 Database / Tcl Starkit (.mk4 / .kit, JL/LJ)".as_ptr(),
    extensions: [c"mk4".as_ptr(), c"kit".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static METALLIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"metallib".as_ptr(),
    mime_type: c"application/x-apple-metallib".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Apple Metal Shader Library (.metallib)".as_ptr(),
    extensions: [c"metallib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static METATRADER_EX4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"metatrader_ex4".as_ptr(),
    mime_type: c"application/x-metatrader-ex4".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"MetaQuotes MetaTrader 4 Compiled MQL4 Expert Advisor Bytecode (.ex4, 2005) MetaQuotes".as_ptr(),
    extensions: [c"ex4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static METATRADER_EX5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"metatrader_ex5".as_ptr(),
    mime_type: c"application/x-metatrader-ex5".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"MetaQuotes MetaTrader 5 Compiled MQL5 Expert Advisor Bytecode (.ex5, 2010) MetaQuotes".as_ptr(),
    extensions: [c"ex5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static METATRADER_HC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"metatrader_hc".as_ptr(),
    mime_type: c"application/x-metatrader-hc".as_ptr(),
    group: c"database".as_ptr(),
    description: c"MetaQuotes MetaTrader 5 History Cache (.hc)".as_ptr(),
    extensions: [c"hc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static METATRADER_SRV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"metatrader_srv".as_ptr(),
    mime_type: c"application/x-metatrader-srv".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MetaQuotes MetaTrader Broker Server Definition (.srv)".as_ptr(),
    extensions: [c"srv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MGSDRV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mgsdrv".as_ptr(),
    mime_type: c"audio/x-msx-mgs".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX MGSDRV Chiptune Music Data (.mgs)".as_ptr(),
    extensions: [c"mgs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MHT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mht".as_ptr(),
    mime_type: c"application/x-mimearchive".as_ptr(),
    group: c"code".as_ptr(),
    description: c"MHTML document".as_ptr(),
    extensions: [c"mht".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MICROCHIP_MPFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"microchip_mpfs".as_ptr(),
    mime_type: c"application/x-microchip-mpfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microchip MPFS2 Embedded HTTP Read-Only Filesystem (.mpfs)".as_ptr(),
    extensions: [c"mpfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MICROFOCUS_GNT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"microfocus_gnt".as_ptr(),
    mime_type: c"application/x-microfocus-gnt".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Micro Focus COBOL Generated Native Code (.gnt, Micro Focus)".as_ptr(),
    extensions: [c"gnt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MICROHOUSE_PROSEC_BIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"microhouse_prosec_bin".as_ptr(),
    mime_type: c"application/x-microhouse-prosec-bin".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Micro House Technical Library Pro-Sec Hardware Database (.bin, 1994) Micro House".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MICROMERITICS_SUBSET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"micromeritics_subset".as_ptr(),
    mime_type: c"application/x-micromeritics-subset".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Micromeritics Instrument Subset Database (.smp / .mth / .anc) Micromeritics".as_ptr(),
    extensions: [c"smp".as_ptr(), c"mth".as_ptr(), c"anc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MIDI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"midi".as_ptr(),
    mime_type: c"audio/midi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Midi".as_ptr(),
    extensions: [c"mid".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"miff".as_ptr(),
    mime_type: c"image/x-miff".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ImageMagick Machine Independent File Format (.miff)".as_ptr(),
    extensions: [c"miff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MIKROTIK_NPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mikrotik_npk".as_ptr(),
    mime_type: c"application/x-mikrotik-npk".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MikroTik RouterOS NPK Firmware Package (.npk)".as_ptr(),
    extensions: [c"npk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MILESTONES_MLB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"milestones_mlb".as_ptr(),
    mime_type: c"application/x-milestones-mlb".as_ptr(),
    group: c"document".as_ptr(),
    description: c"KIDASA Software Milestones Professional Schedule (.mlb / .mlq) KIDASA Software".as_ptr(),
    extensions: [c"mlb".as_ptr(), c"mlq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MINDWARE_MFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mindware_mfs".as_ptr(),
    mime_type: c"application/x-mindware-mfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Mindware Studios Meng Engine Archive (.mfs)".as_ptr(),
    extensions: [c"mfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MINIDUMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"minidump".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows minidump".as_ptr(),
    extensions: [c"dmp".as_ptr(), c"mdmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MINITAUR_LOG_PROTO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"minitaur_log_proto".as_ptr(),
    mime_type: c"application/x-minitaur-log-proto".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Minitaur Robotics Latency Telemetry Protobuf Log (.proto)".as_ptr(),
    extensions: [c"proto".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MIR_MAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mir_map".as_ptr(),
    mime_type: c"application/x-mir-map".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"WeMade Entertainment The Legend of Mir 2 Tile Map (.map, 2001) WeMade".as_ptr(),
    extensions: [c"map".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MKJAMZ_MKJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mkjamz_mkj".as_ptr(),
    mime_type: c"audio/x-mkj".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MKJamz AdLib FM Module (.mkj)".as_ptr(),
    extensions: [c"mkj".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MKV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mkv".as_ptr(),
    mime_type: c"video/x-matroska".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Matroska".as_ptr(),
    extensions: [c"mkv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MLAT_MAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mlat_mad".as_ptr(),
    mime_type: c"audio/x-mad".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Søren Lund MLAT AdLib Tracker Module (.mad)".as_ptr(),
    extensions: [c"mad".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MLOCATE_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mlocate_db".as_ptr(),
    mime_type: c"application/x-mlocate-db".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Linux mlocate File System Index Database (mlocate.db)".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MM_GEOV1_R565: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mm_geov1_r565".as_ptr(),
    mime_type: c"application/x-mm-geov1-r565".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Magneti Marelli RT6/SMEG Automotive Navigation GEOV1 RGB565 Image (.r565) Magneti Marelli".as_ptr(),
    extensions: [c"r565".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MMCMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mmcmp".as_ptr(),
    mime_type: c"application/x-mmcmp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Emmanuel Giasson Music Module Compressor (.mmcmp / .zig)".as_ptr(),
    extensions: [c"mmcmp".as_ptr(), c"zig".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MMD_PMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mmd_pmd".as_ptr(),
    mime_type: c"model/x-mmd-pmd".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"MikuMikuDance Polygon Model Data (.pmd, 2008) Yu Higuchi".as_ptr(),
    extensions: [c"pmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MMD_PMX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mmd_pmx".as_ptr(),
    mime_type: c"model/x-mmd-pmx".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"MikuMikuDance Polygon Model eXtended (.pmx, 2011) Yu Higuchi".as_ptr(),
    extensions: [c"pmx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MMD_VMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mmd_vmd".as_ptr(),
    mime_type: c"application/x-mmd-vmd".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"MikuMikuDance Vocaloid Motion Data (.vmd, 2008) Yu Higuchi".as_ptr(),
    extensions: [c"vmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MO3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mo3".as_ptr(),
    mime_type: c"audio/x-mo3".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Un4seen Developments MO3 Compressed Tracker Module (.mo3)".as_ptr(),
    extensions: [c"mo3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOBICORE_MCLF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mobicore_mclf".as_ptr(),
    mime_type: c"application/x-mobicore-mclf".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Trustonic Kinibi / G&D MobiCore TEE Loadable Binary (MCLF)".as_ptr(),
    extensions: [c"mclf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOM_GRF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mom_grf".as_ptr(),
    mime_type: c"application/x-mom-grf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Simtex Master of Magic Graphics Resource (.grf, 1994) Simtex / MicroProse".as_ptr(),
    extensions: [c"grf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MONOTONE_MON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"monotone_mon".as_ptr(),
    mime_type: c"audio/x-monotone".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Trixter MONOTONE PC Speaker / OPL Tracker Module (.mon)".as_ptr(),
    extensions: [c"mon".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOOF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"moof".as_ptr(),
    mime_type: c"application/x-moof".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Applesauce Macintosh MOOF Flux / Bitstream Floppy Image (.moof)".as_ptr(),
    extensions: [c"moof".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOONBLASTER_MBM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"moonblaster_mbm".as_ptr(),
    mime_type: c"audio/x-mbm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Remco Schrijvers MSX MoonBlaster FM/Audio Module (.mbm)".as_ptr(),
    extensions: [c"mbm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MORE_OUTLINE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"more_outline".as_ptr(),
    mime_type: c"application/x-more-outline".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Living Videotext / Symantec MORE 2.0/3.0 Macintosh Outline (.more, MOR2 / MOR3 / MORE)".as_ptr(),
    extensions: [c"more".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOSHIDRAW_YZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"moshidraw_yz".as_ptr(),
    mime_type: c"application/x-moshidraw-yz".as_ptr(),
    group: c"document".as_ptr(),
    description: c"MoshiDraw / WINYZ Laser Engraving Design v6.0 (.yz)".as_ptr(),
    extensions: [c"yz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOTOROLA_MOTOLOGO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"motorola_motologo".as_ptr(),
    mime_type: c"application/x-motorola-motologo".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Motorola MotoLogo Boot Logo Container (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOTOROLA_PIV_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"motorola_piv_img".as_ptr(),
    mime_type: c"application/x-motorola-piv-image".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Motorola PIV Signed Partition Image (MOT_PIV_FULL256)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOUSEWRITE_DOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mousewrite_doc".as_ptr(),
    mime_type: c"application/x-mousewrite".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Roger Wagner MouseWrite Apple IIe/IIc/IIGS Word Processor Document (.mw)".as_ptr(),
    extensions: [c"mw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOXA_FRM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"moxa_frm".as_ptr(),
    mime_type: c"application/x-moxa-frm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Moxa Industrial Device Firmware Image (.frm)".as_ptr(),
    extensions: [c"frm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MOZLZ4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mozlz4".as_ptr(),
    mime_type: c"application/x-mozlz4".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Mozilla Firefox LZ4-Compressed JSON Container (.jsonlz4 / .mozlz4)".as_ptr(),
    extensions: [c"jsonlz4".as_ptr(), c"mozlz4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MP2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mp2".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"MP2 stream".as_ptr(),
    extensions: [c"mp2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MP3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mp3".as_ptr(),
    mime_type: c"audio/mpeg".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MP3 media file".as_ptr(),
    extensions: [c"mp3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MP4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mp4".as_ptr(),
    mime_type: c"video/mp4".as_ptr(),
    group: c"video".as_ptr(),
    description: c"MP4 media file".as_ptr(),
    extensions: [c"mp4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MPEGTS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mpegts".as_ptr(),
    mime_type: c"video/MP2T".as_ptr(),
    group: c"video".as_ptr(),
    description: c"MPEG Transport stream".as_ptr(),
    extensions: [c"ts".as_ptr(), c"tsv".as_ptr(), c"tsa".as_ptr(), c"m2t".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MPQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mpq".as_ptr(),
    mime_type: c"application/x-blizzard-mpq".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Blizzard MoPaQ Archive (.mpq)".as_ptr(),
    extensions: [c"mpq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MPU401_MTK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mpu401_mtk".as_ptr(),
    mime_type: c"audio/x-mtk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MPU-401 Tracker Compressed Module (.mtk)".as_ptr(),
    extensions: [c"mtk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MRC_MAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mrc_map".as_ptr(),
    mime_type: c"image/x-mrc".as_ptr(),
    group: c"image".as_ptr(),
    description: c"MRC / CCP4 3D Electron Microscopy & Crystallography Density Map (.mrc / .map, 1982) MRC LMB / CCP4".as_ptr(),
    extensions: [c"mrc".as_ptr(), c"map".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MRUBY_RITE_MRB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mruby_rite_mrb".as_ptr(),
    mime_type: c"application/x-mruby-bytecode".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"mruby RiteVM Compiled Bytecode (.mrb)".as_ptr(),
    extensions: [c"mrb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_ADVISOR_HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_advisor_hlp".as_ptr(),
    mime_type: c"application/x-ms-advisor-help".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft QuickHelp / Advisor Database (.hlp, 1988) Microsoft".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_ANSWER_WIZARD_AW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_answer_wizard_aw".as_ptr(),
    mime_type: c"application/x-ms-answer-wizard-aw".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Office Answer Wizard Database (.aw)".as_ptr(),
    extensions: [c"aw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_CERT_STORE_SST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_cert_store_sst".as_ptr(),
    mime_type: c"application/x-ms-cert-store-sst".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Serialized Certificate Store (.sst, x00x00x00x00CERT)".as_ptr(),
    extensions: [c"sst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_NETMON_NPB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_netmon_npb".as_ptr(),
    mime_type: c"application/x-ms-netmon-npb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Network Monitor 3 Compiled Parser (.npb)".as_ptr(),
    extensions: [c"npb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_PRAGUE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_prague".as_ptr(),
    mime_type: c"application/x-ms-prague".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Fluid Framework / Loop Prague Binary Serialization (.loop / .fluid)".as_ptr(),
    extensions: [c"loop".as_ptr(), c"fluid".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_PUBLISHER_BORDERART_BDR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_publisher_borderart_bdr".as_ptr(),
    mime_type: c"application/x-ms-publisher-borderart-bdr".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Publisher BorderArt File (.bdr, 1993) Microsoft".as_ptr(),
    extensions: [c"bdr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_PUBLISHER_PAGEWIZARD_POC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_publisher_pagewizard_poc".as_ptr(),
    mime_type: c"application/x-ms-publisher-pagewizard-poc".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Publisher PageWizard Design Assistant (.poc, 1995) Microsoft".as_ptr(),
    extensions: [c"poc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_PUBLISHER_PREVIEW_PRV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_publisher_preview_prv".as_ptr(),
    mime_type: c"application/x-ms-publisher-preview-prv".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Publisher Template Preview (.prv, 1995) Microsoft".as_ptr(),
    extensions: [c"prv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_SETUP_TDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_setup_tdf".as_ptr(),
    mime_type: c"application/x-ms-setup-tdf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Setup Font/Typeface Definition (SETUP.TDF, 1991) Microsoft".as_ptr(),
    extensions: [c"tdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_WINDOWS_BOOT_LOGO_LGO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_windows_boot_logo_lgo".as_ptr(),
    mime_type: c"application/x-ms-windows-boot-logo-lgo".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Microsoft Windows 3.x EGA/VGA Startup Logo Code Stub (.lgo, 1990) Microsoft".as_ptr(),
    extensions: [c"lgo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MS_WINDOWS_POSTSCRIPT_WPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ms_windows_postscript_wpd".as_ptr(),
    mime_type: c"application/x-ms-windows-postscript-wpd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows 3.1 PostScript Printer Description (.wpd, 1992) Microsoft".as_ptr(),
    extensions: [c"wpd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSCOMPRESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mscompress".as_ptr(),
    mime_type: c"application/x-ms-compress-szdd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MS Compress archive data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSDELTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msdelta".as_ptr(),
    mime_type: c"application/x-ms-delta".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Windows Update Delta Compression Patch (PA30 / PA19)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSDOS_COUNTRY_SYS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msdos_country_sys".as_ptr(),
    mime_type: c"application/x-msdos-country-sys".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS-DOS / PC DOS Country Information Table (COUNTRY.SYS, 1987) Microsoft / IBM".as_ptr(),
    extensions: [c"sys".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSDOS_KEYBOARD_SYS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msdos_keyboard_sys".as_ptr(),
    mime_type: c"application/x-msdos-keyboard-sys".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS-DOS / PC DOS Keyboard Layout Table (KEYBOARD.SYS, 1987) Microsoft / IBM".as_ptr(),
    extensions: [c"sys".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSFS_BGL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msfs_bgl".as_ptr(),
    mime_type: c"application/x-msfs-bgl".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Flight Simulator BGL Scenery Database (.bgl, 2003) Microsoft".as_ptr(),
    extensions: [c"bgl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSFS_SPB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msfs_spb".as_ptr(),
    mime_type: c"application/x-msfs-spb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Flight Simulator SimProp Binary XML (.spb, 2006) Microsoft".as_ptr(),
    extensions: [c"spb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSGPACK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msgpack".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MessagePack".as_ptr(),
    extensions: [c"mpk".as_ptr(), c"msgpack".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msi".as_ptr(),
    mime_type: c"application/x-msi".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Installer file".as_ptr(),
    extensions: [c"msi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSIE_INDEX_DAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msie_index_dat".as_ptr(),
    mime_type: c"application/x-msie-cache".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Internet Explorer WinINet URL Cache (index.dat)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSIX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msix".as_ptr(),
    mime_type: c"application/msix".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows app package".as_ptr(),
    extensions: [c"msix".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSP_PAINT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msp_paint".as_ptr(),
    mime_type: c"image/x-msp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Microsoft Paint v1/v2 bitmap (.msp)".as_ptr(),
    extensions: [c"msp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSSSCARD_AX_LOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mssscard_ax_log".as_ptr(),
    mime_type: c"application/x-mssscard-ax-log".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft SmartCard Provider Trace Record (.ax)".as_ptr(),
    extensions: [c"ax".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mst".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"mst".as_ptr(),
    extensions: [c"mst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSTOR_IBX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mstor_ibx".as_ptr(),
    mime_type: c"application/x-mstor-ibx".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"mstor Indexed Mailbox Archive (.ibx)".as_ptr(),
    extensions: [c"ibx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSTS_SIMIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msts_simis".as_ptr(),
    mime_type: c"application/x-msts-simis".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Train Simulator Kuju SIMIS Binary Data (.s / .ace, SIMISA@F)".as_ptr(),
    extensions: [c"s".as_ptr(), c"ace".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSVC_IFC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msvc_ifc".as_ptr(),
    mime_type: c"application/x-msvc-ifc".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Microsoft Visual C++ C++20 Built Module Interface (.ifc)".as_ptr(),
    extensions: [c"ifc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSVC_IPDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msvc_ipdb".as_ptr(),
    mime_type: c"application/x-msvc-ipdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Visual C++ Incremental Linker Program Database (.ipdb)".as_ptr(),
    extensions: [c"ipdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSWORKS_WDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msworks_wdb".as_ptr(),
    mime_type: c"application/vnd.ms-works".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Works Database v1-2 (.wdb / .bdb)".as_ptr(),
    extensions: [c"wdb".as_ptr(), c"bdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSX_CAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msx_cas".as_ptr(),
    mime_type: c"application/x-msx-cas".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MSX Emulator Cassette Tape Image (.cas)".as_ptr(),
    extensions: [c"cas".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSX_FAC_MUS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msx_fac_mus".as_ptr(),
    mime_type: c"audio/x-msx-fac".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX FAC SoundTracker Music Module (.mus / .sm1 / .sm2)".as_ptr(),
    extensions: [c"mus".as_ptr(), c"sm1".as_ptr(), c"sm2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MSX_MPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"msx_mpk".as_ptr(),
    mime_type: c"audio/x-msx-mpk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX K-kaz MPK Chiptune Music Data (.mpk)".as_ptr(),
    extensions: [c"mpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MT2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mt2".as_ptr(),
    mime_type: c"audio/x-mt2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Yannick Delwiche MadTracker 2 Module (.mt2)".as_ptr(),
    extensions: [c"mt2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MTC_CHIPTUNE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mtc_chiptune".as_ptr(),
    mime_type: c"audio/x-mtc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MTC Chiptune Module (.mtc)".as_ptr(),
    extensions: [c"mtc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MTL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mtl".as_ptr(),
    mime_type: c"model/mtl".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Wavefront material library".as_ptr(),
    extensions: [c"mtl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MULTIMATE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"multimate".as_ptr(),
    mime_type: c"application/x-multimate".as_ptr(),
    group: c"document".as_ptr(),
    description: c"MultiMate Text Document (.dox, PRONOM x-fmt/347)".as_ptr(),
    extensions: [c"dox".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MULTIPLAN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"multiplan".as_ptr(),
    mime_type: c"application/x-multiplan".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Multiplan Spreadsheet (.mp / .mod)".as_ptr(),
    extensions: [c"mp".as_ptr(), c"mod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MUM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mum".as_ptr(),
    mime_type: c"text/xml".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Update Package file".as_ptr(),
    extensions: [c"mum".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static MUSIC_STUDIO_SNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"music_studio_sng".as_ptr(),
    mime_type: c"audio/x-sng".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Activision The Music Studio Song (.sng)".as_ptr(),
    extensions: [c"sng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MUSICAL_ENLIGHTENMENT_TME: MagikaTypeInfo = MagikaTypeInfo {
    label: c"musical_enlightenment_tme".as_ptr(),
    mime_type: c"audio/x-musical-enlightenment".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"N. J. Luuring Jr. The Musical Enlightenment Amiga Module (.tme)".as_ptr(),
    extensions: [c"tme".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MUSICLINE_ML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"musicline_ml".as_ptr(),
    mime_type: c"audio/x-musicline".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Conny & Christian Cyrus Musicline Editor Amiga Module (.ml, MLEDMODL)".as_ptr(),
    extensions: [c"ml".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MVS_OBJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mvs_obj".as_ptr(),
    mime_type: c"application/x-ibm-mvs-obj".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"IBM System/370 & z/OS MVS Object Deck (ESD/TXT/RLD/END)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MVS_TRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mvs_tracker".as_ptr(),
    mime_type: c"audio/x-mus".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MVS Tracker Module (.mus)".as_ptr(),
    extensions: [c"mus".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MVX_MVM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mvx_mvm".as_ptr(),
    mime_type: c"audio/x-mvm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MVX Tracker Module (.mvm)".as_ptr(),
    extensions: [c"mvm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MXMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mxmf".as_ptr(),
    mime_type: c"audio/mobile-xmf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Mobile eXtensible Music Format (.mxmf, PRONOM fmt/961, LoC fdd000121)".as_ptr(),
    extensions: [c"mxmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MXTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mxtx".as_ptr(),
    mime_type: c"audio/x-mxtx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Taliesin Amiga MaxTrax Module (.mxtx)".as_ptr(),
    extensions: [c"mxtx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static MYSQL_STORAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"mysql_storage".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MySQL / MariaDB storage".as_ptr(),
    extensions: [c"ibd".as_ptr(), c"myd".as_ptr(), c"myi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static N64_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"n64_rom".as_ptr(),
    mime_type: c"application/x-n64-rom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo 64 ROM Image (.z64 / .n64 / .v64)".as_ptr(),
    extensions: [c"z64".as_ptr(), c"n64".as_ptr(), c"v64".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NAF_BIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"naf_bio".as_ptr(),
    mime_type: c"application/x-naf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nucleotide Archival Format (.naf)".as_ptr(),
    extensions: [c"naf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NAMCO_HBSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"namco_hbss".as_ptr(),
    mime_type: c"application/x-namco-hbss".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Bandai Namco Smash Bros. Ultimate NuLibrary Binary Resource (.nuanmb / .numatb / .numshb / .numdlb)".as_ptr(),
    extensions: [c"nuanmb".as_ptr(), c"numatb".as_ptr(), c"numshb".as_ptr(), c"numdlb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NAMX_PROFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"namx_profile".as_ptr(),
    mime_type: c"application/x-namx-profile".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NAMX Neural Amp Modeler Extended Profile (.namx)".as_ptr(),
    extensions: [c"namx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NASM_RDOFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nasm_rdoff".as_ptr(),
    mime_type: c"application/x-rdoff".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"NASM Relocatable Dynamic Object File Format (.rdf, RDOFF2)".as_ptr(),
    extensions: [c"rdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NBA2K_CMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nba2k_cmg".as_ptr(),
    mime_type: c"application/x-nba2k-cmg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"NBA 2K Career Game Save / Mod Archive (.cmg)".as_ptr(),
    extensions: [c"cmg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NBI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nbi".as_ptr(),
    mime_type: c"application/x-nbi-legacy".as_ptr(),
    group: c"document".as_ptr(),
    description: c"NBI Legacy Word Processing Document (.leg)".as_ptr(),
    extensions: [c"leg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NC_MSG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nc_msg".as_ptr(),
    mime_type: c"application/x-norton-msg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Peter Norton Computing Norton Commander Message File (.msg)".as_ptr(),
    extensions: [c"msg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NCSTUDIO_DYN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ncstudio_dyn".as_ptr(),
    mime_type: c"application/x-ncstudio-dyn".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Weihong NcStudio CNC Dynamic State File (.dyn)".as_ptr(),
    extensions: [c"dyn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NDIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ndif".as_ptr(),
    mime_type: c"application/x-apple-diskimage".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Apple DiskCopy 6.0 New Disk Image Format (.dmg / .img)".as_ptr(),
    extensions: [c"dmg".as_ptr(), c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NDS_NSBTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nds_nsbtx".as_ptr(),
    mime_type: c"image/x-nds-nsbtx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Nintendo DS Nitro SDK 3D Binary Texture (.nsbtx, 2004) Nintendo".as_ptr(),
    extensions: [c"nsbtx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ned".as_ptr(),
    mime_type: c"audio/x-ned".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bananmos NES NerdTracker II Module (.ned)".as_ptr(),
    extensions: [c"ned".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NEOGEO_ABKCPUR_SRM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"neogeo_abkcpur_srm".as_ptr(),
    mime_type: c"application/x-neogeo-abkcpur-srm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Neo Geo / FBNeo ABKCPUR MAO Battery Save RAM (.srm)".as_ptr(),
    extensions: [c"srm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NES_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nes_rom".as_ptr(),
    mime_type: c"application/x-nes-rom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Entertainment System / Famicom ROM (.nes / iNES / NES 2.0)".as_ptr(),
    extensions: [c"nes".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NETCDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"netcdf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"NetCDF".as_ptr(),
    extensions: [c"cdf".as_ptr(), c"nc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NETGEAR_CHK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"netgear_chk".as_ptr(),
    mime_type: c"application/x-netgear-chk".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NETGEAR CHK Router Firmware Image (.chk)".as_ptr(),
    extensions: [c"chk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NETRON_NFW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"netron_nfw".as_ptr(),
    mime_type: c"application/x-netron-nfw".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Obsidian Control Systems Netron DMX Node Firmware Image (.nfw) Obsidian Control Systems".as_ptr(),
    extensions: [c"nfw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NEWWORD_OVR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"newword_ovr".as_ptr(),
    mime_type: c"application/x-newword-ovr".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NewStar NewWord 2 Messages Overlay (.ovr)".as_ptr(),
    extensions: [c"ovr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NEXON_KARTRIDER_1S: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nexon_kartrider_1s".as_ptr(),
    mime_type: c"application/x-nexon-kartrider-1s".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Nexon Crazyracing KartRider 3D Model (.1s)".as_ptr(),
    extensions: [c"1s".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NEXTSTEP_WMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nextstep_wmd".as_ptr(),
    mime_type: c"application/x-nextstep-wmd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NeXTSTEP Workspace Manager Window Attributes (.wmd, binary, 1989 NeXT)".as_ptr(),
    extensions: [c"wmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NEXUS_IMGDATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nexus_imgdata".as_ptr(),
    mime_type: c"application/x-nexus-imgdata".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google Nexus IMGDATA! Boot Splash Container (.img)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NFDUMP_FLOW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nfdump_flow".as_ptr(),
    mime_type: c"application/x-nfdump".as_ptr(),
    group: c"application".as_ptr(),
    description: c"nfdump NetFlow / IPFIX Binary Capture (nfcapd, 0xA50C)".as_ptr(),
    extensions: [c"nf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NGP_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ngp_rom".as_ptr(),
    mime_type: c"application/x-neo-geo-pocket-rom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SNK Neo Geo Pocket / Pocket Color Cartridge ROM (.ngp / .ngc)".as_ptr(),
    extensions: [c"ngp".as_ptr(), c"ngc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NIBTOOLS_NIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nibtools_nib".as_ptr(),
    mime_type: c"application/x-c64-nib".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Commodore 1541 Nibtools Raw/Compressed GCR Disk Image (.nib / .nbz)".as_ptr(),
    extensions: [c"nib".as_ptr(), c"nbz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nif".as_ptr(),
    mime_type: c"model/x-nif".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"NetImmerse / Gamebryo 3D Model (.nif)".as_ptr(),
    extensions: [c"nif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NIFTI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nifti".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"NIfTI neuroimaging data".as_ptr(),
    extensions: [c"nii".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_AAMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_aamp".as_ptr(),
    mime_type: c"application/x-nintendo-aamp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nintendo EPD Binary Parameter Archive (.bxml / .baatarc / .aamp, 2015) Nintendo".as_ptr(),
    extensions: [c"bxml".as_ptr(), c"baatarc".as_ptr(), c"aamp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_AINB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_ainb".as_ptr(),
    mime_type: c"application/x-nintendo-ainb".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Nintendo AI Node Binary Graph (.ainb, 2022) Nintendo".as_ptr(),
    extensions: [c"ainb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_AST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_ast".as_ptr(),
    mime_type: c"audio/x-nintendo-ast".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo GameCube / Wii AST Audio Stream (.ast)".as_ptr(),
    extensions: [c"ast".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_BMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_bmg".as_ptr(),
    mime_type: c"application/x-nintendo-bmg".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Nintendo JSystem Binary Message Group (.bmg, 2001) Nintendo".as_ptr(),
    extensions: [c"bmg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_BNTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_bntx".as_ptr(),
    mime_type: c"image/x-nintendo-bntx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Nintendo Switch Binary Texture Container (.bntx)".as_ptr(),
    extensions: [c"bntx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_BWAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_bwav".as_ptr(),
    mime_type: c"audio/x-nintendo-bwav".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo Switch / Wii U Binary Wave Audio (.bwav)".as_ptr(),
    extensions: [c"bwav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_BYML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_byml".as_ptr(),
    mime_type: c"application/x-nintendo-byml".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nintendo Binary YAML (.byml / .byaml / .bgyml, 2015) Nintendo".as_ptr(),
    extensions: [c"byml".as_ptr(), c"byaml".as_ptr(), c"bgyml".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_font".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Nintendo NW4R / NW4C Binary Font Resource (RFNT / CFNT / FFNT)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_MSBT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_msbt".as_ptr(),
    mime_type: c"application/x-nintendo-msbt".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Nintendo LMS Binary Message Table / Flow / Project (.msbt / .msbf / .msbp, 2010) Nintendo".as_ptr(),
    extensions: [c"msbt".as_ptr(), c"msbf".as_ptr(), c"msbp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_NARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_narc".as_ptr(),
    mime_type: c"application/x-nintendo-narc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo DS Nitro Archive (.narc / .carc, NARC)".as_ptr(),
    extensions: [c"narc".as_ptr(), c"carc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_U8: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_u8".as_ptr(),
    mime_type: c"application/x-nintendo-u8".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Wii / Wii U U8 Content Archive (.arc / .szs, U\\\\xAA8-)".as_ptr(),
    extensions: [c"arc".as_ptr(), c"szs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_WII_BNR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_wii_bnr".as_ptr(),
    mime_type: c"application/x-nintendo-wii-bnr".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nintendo Wii IMET Opening Banner (.bnr)".as_ptr(),
    extensions: [c"bnr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NINTENDO_WIIU_GTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nintendo_wiiu_gtx".as_ptr(),
    mime_type: c"image/x-nintendo-wiiu-gtx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Nintendo Wii U GX2 Texture Container (.gtx)".as_ptr(),
    extensions: [c"gtx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NIST_SPHERE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nist_sphere".as_ptr(),
    mime_type: c"audio/x-nist-sphere".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NIST SPHERE Speech Header Resources Audio (.sph / .wav)".as_ptr(),
    extensions: [c"sph".as_ptr(), c"wav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NITROPLUS_NPA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nitroplus_npa".as_ptr(),
    mime_type: c"application/x-nitroplus-npa".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nitroplus Visual Novel Engine NPA Archive (.npa, 2004) Nitroplus".as_ptr(),
    extensions: [c"npa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NMS_MBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nms_mbin".as_ptr(),
    mime_type: c"application/x-nms-mbin".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Hello Games No Man's Sky Compiled EXML Binary (.mbin)".as_ptr(),
    extensions: [c"mbin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOKIA_BB5_PPM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nokia_bb5_ppm".as_ptr(),
    mime_type: c"application/x-nokia-bb5-ppm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Nokia BB5 PPM Language Pack Firmware (.ppm)".as_ptr(),
    extensions: [c"ppm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOKIA_NGG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nokia_ngg".as_ptr(),
    mime_type: c"image/x-nokia-ngg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Nokia Group Graphics Bitmap (.ngg)".as_ptr(),
    extensions: [c"ngg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOKIA_NLM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nokia_nlm".as_ptr(),
    mime_type: c"image/x-nokia-nlm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Nokia Logo Manager Bitmap (.nlm)".as_ptr(),
    extensions: [c"nlm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOKIA_NPM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nokia_npm".as_ptr(),
    mime_type: c"image/x-nokia-npm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Nokia Picture Message Bitmap (.npm)".as_ptr(),
    extensions: [c"npm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NORTON_GUIDES_HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"norton_guides_hlp".as_ptr(),
    mime_type: c"application/x-norton-guides-hlp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Peter Norton Computing Norton Guides Help Database (.ng / .hlp, 1987) Peter Norton Computing".as_ptr(),
    extensions: [c"ng".as_ptr(), c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOS_INSTALLER_DATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nos_installer_data".as_ptr(),
    mime_type: c"application/x-nos-installer-data".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Netopsystems FEAD Optimizer (NOS_PO) Archive".as_ptr(),
    extensions: [c"nos".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOTATOR_SONG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"notator_song".as_ptr(),
    mime_type: c"audio/x-notator-song".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"C-Lab Notator / Creator MIDI song arrangement".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOVELL_NETWARE_MESSAGE_MSG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"novell_netware_message_msg".as_ptr(),
    mime_type: c"application/x-novell-netware-message-msg".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Novell NetWare / Message Librarian Message Catalog (.msg, 1989) Novell".as_ptr(),
    extensions: [c"msg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOVELL_NETWARE_RPL_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"novell_netware_rpl_rom".as_ptr(),
    mime_type: c"application/x-novell-netware-rpl-rom".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Novell NetWare Remote Program Load (RPL) Boot ROM (.rom / .rpl, 1989) Novell".as_ptr(),
    extensions: [c"rom".as_ptr(), c"rpl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NOVELL_SYS_HELP_HEP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"novell_sys_help_hep".as_ptr(),
    mime_type: c"application/x-novell-sys-help-hep".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Novell NetWare SYS$HELP Help File (.hep, 1989) Novell".as_ptr(),
    extensions: [c"hep".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NPY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"npy".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Numpy Array".as_ptr(),
    extensions: [c"npy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NPZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"npz".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Numpy Arrays Archive".as_ptr(),
    extensions: [c"npz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NRRD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nrrd".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"Nearly Raw Raster Data".as_ptr(),
    extensions: [c"nhdr".as_ptr(), c"nrrd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nsf".as_ptr(),
    mime_type: c"audio/x-nsf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Nintendo Entertainment System Sound Format (.nsf / .nsfe)".as_ptr(),
    extensions: [c"nsf".as_ptr(), c"nsfe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NSIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nsis".as_ptr(),
    mime_type: c"application/x-nsis".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nullsoft Scriptable Install System (NSIS) data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NTFSCLONE_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ntfsclone_img".as_ptr(),
    mime_type: c"application/x-ntfsclone".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Linux-NTFS ntfsclone Sparse Partition Backup Image (\\0ntfsclone-image)".as_ptr(),
    extensions: [c"ntfsclone".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NTSP_SYSTEM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ntsp_system".as_ptr(),
    mime_type: c"audio/x-ntsp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga NTSP-system Audio Module (.ntsp)".as_ptr(),
    extensions: [c"ntsp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NUPKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nupkg".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"NuGet Package".as_ptr(),
    extensions: [c"nupkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NUPPELVIDEO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nuppelvideo".as_ptr(),
    mime_type: c"video/x-nuv".as_ptr(),
    group: c"video".as_ptr(),
    description: c"NuppelVideo / MythTV RTjpeg+LZO Television Capture (.nuv)".as_ptr(),
    extensions: [c"nuv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static NUT_VIDEO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"nut_video".as_ptr(),
    mime_type: c"video/x-nut".as_ptr(),
    group: c"video".as_ptr(),
    description: c"FFmpeg / MPlayer NUT Open Multimedia Container (.nut)".as_ptr(),
    extensions: [c"nut".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OBARSKI_OSP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"obarski_osp".as_ptr(),
    mime_type: c"audio/x-obarski-osp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Karsten Obarski Synth Pack Bank (.osp)".as_ptr(),
    extensions: [c"osp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OBERON_ODC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"oberon_odc".as_ptr(),
    mime_type: c"application/x-oberon-odc".as_ptr(),
    group: c"document".as_ptr(),
    description: c"BlackBox Component Builder / Oberon-2 Compound Document (.odc, 1997) Oberon microsystems".as_ptr(),
    extensions: [c"odc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OBJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"obj".as_ptr(),
    mime_type: c"model/obj".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Wavefront OBJ".as_ptr(),
    extensions: [c"obj".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static OBJECTIVEC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"objectivec".as_ptr(),
    mime_type: c"text/x-objcsrc".as_ptr(),
    group: c"code".as_ptr(),
    description: c"ObjectiveC source".as_ptr(),
    extensions: [c"m".as_ptr(), c"mm".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static OCAML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ocaml".as_ptr(),
    mime_type: c"text/ocaml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"OCaml".as_ptr(),
    extensions: [c"ml".as_ptr(), c"mli".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static OCAML_CMI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ocaml_cmi".as_ptr(),
    mime_type: c"application/x-ocaml-object".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"OCaml Compiled Interface & Bytecode Object (.cmi / .cmo / .cma / .cmx / .cmt)".as_ptr(),
    extensions: [c"cmi".as_ptr(), c"cmo".as_ptr(), c"cma".as_ptr(), c"cmx".as_ptr(), c"cmt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OCDBT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ocdbt".as_ptr(),
    mime_type: c"application/x-tensorstore-ocdbt".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Google TensorStore OCDBT B+Tree Node / Manifest (.ocdbt)".as_ptr(),
    extensions: [c"ocdbt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ODEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"odex".as_ptr(),
    mime_type: c"application/x-executable-elf".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"ODEX ELF executable".as_ptr(),
    extensions: [c"odex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ODP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"odp".as_ptr(),
    mime_type: c"application/vnd.oasis.opendocument.presentation".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OpenDocument Presentation".as_ptr(),
    extensions: [c"odp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ODS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ods".as_ptr(),
    mime_type: c"application/vnd.oasis.opendocument.spreadsheet".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OpenDocument Spreadsheet".as_ptr(),
    extensions: [c"ods".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ODT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"odt".as_ptr(),
    mime_type: c"application/vnd.oasis.opendocument.text".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OpenDocument Text".as_ptr(),
    extensions: [c"odt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OFFICE_OWNER_LOCK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"office_owner_lock".as_ptr(),
    mime_type: c"application/x-office-owner-lock".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Word / Office Owner Lock File (~$*.doc / ~$*.docx)".as_ptr(),
    extensions: [c"doc".as_ptr(), c"docx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OFR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ofr".as_ptr(),
    mime_type: c"audio/x-optimfrog".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"OptimFROG Lossless Audio (.ofr / .ofs)".as_ptr(),
    extensions: [c"ofr".as_ptr(), c"ofs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OGG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ogg".as_ptr(),
    mime_type: c"audio/ogg".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ogg data".as_ptr(),
    extensions: [c"ogg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OGL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ogl".as_ptr(),
    mime_type: c"image/x-ogl".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ObjectGraphics OGL Drawing (.ogl)".as_ptr(),
    extensions: [c"ogl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OGRE_MESH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ogre_mesh".as_ptr(),
    mime_type: c"model/x-ogre-mesh".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"OGRE 3D Engine Binary Mesh (.mesh)".as_ptr(),
    extensions: [c"mesh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OMEGA_LOGIC_CORK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"omega_logic_cork".as_ptr(),
    mime_type: c"application/x-omega-logic-cork".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Omega Logic CORK Compressed Installer File (.dl$ / .ex$)".as_ptr(),
    extensions: [c"dl".as_ptr(), c"ex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"omf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"OMF object".as_ptr(),
    extensions: [c"obj".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OMF_ONYX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"omf_onyx".as_ptr(),
    mime_type: c"audio/x-omf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Onyx Music File (.omf)".as_ptr(),
    extensions: [c"omf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ONE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"one".as_ptr(),
    mime_type: c"application/msonenote".as_ptr(),
    group: c"document".as_ptr(),
    description: c"OneNote".as_ptr(),
    extensions: [c"one".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ONEDRIVE_ODL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"onedrive_odl".as_ptr(),
    mime_type: c"application/x-onedrive-odl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft OneDrive Obfuscated Diagnostic Log (.odl)".as_ptr(),
    extensions: [c"odl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ONETOC2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"onetoc2".as_ptr(),
    mime_type: c"application/onenote".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft OneNote Table of Contents v2 (.onetoc2)".as_ptr(),
    extensions: [c"onetoc2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ONNX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"onnx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Open Neural Network Exchange".as_ptr(),
    extensions: [c"onnx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OPENFST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"openfst".as_ptr(),
    mime_type: c"application/x-openfst".as_ptr(),
    group: c"database".as_ptr(),
    description: c"OpenFst Weighted Finite-State Transducer / Archive (.fst / .far)".as_ptr(),
    extensions: [c"fst".as_ptr(), c"far".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OPENRCT2_PARK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"openrct2_park".as_ptr(),
    mime_type: c"application/x-openrct2-park".as_ptr(),
    group: c"application".as_ptr(),
    description: c"OpenRCT2 Park Save / Scenario Container (.park, 2021) OpenRCT2".as_ptr(),
    extensions: [c"park".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OPENSSH_KRL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"openssh_krl".as_ptr(),
    mime_type: c"application/x-openssh-krl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"OpenSSH Key Revocation List (.krl, SSHKRL\\\\n\\\\x00)".as_ptr(),
    extensions: [c"krl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OPENVINO_IR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"openvino_ir".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"OpenVINO IR".as_ptr(),
    extensions: [c"xml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static OPENZIM_ZIM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"openzim_zim".as_ptr(),
    mime_type: c"application/x-zim".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"openZIM Offline Wiki Content Archive (.zim, ZIM\\\\x04)".as_ptr(),
    extensions: [c"zim".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OPX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"opx".as_ptr(),
    mime_type: c"audio/x-opx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX / DOS OPLLDRV FM Chiptune (.opx)".as_ptr(),
    extensions: [c"opx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ORACLE_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"oracle_db".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Oracle database / export".as_ptr(),
    extensions: [c"dbf".as_ptr(), c"dmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ORC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"orc".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"Apache ORC".as_ptr(),
    extensions: [c"orc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ORG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"org".as_ptr(),
    mime_type: c"audio/x-org".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Studio Pixel Cave Story Organya Music (.org)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ORIC_DSK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"oric_dsk".as_ptr(),
    mime_type: c"application/x-oric-dsk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Oric Atmos / Telestrat MFM Disk Image (.dsk)".as_ptr(),
    extensions: [c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OS2_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"os2_font".as_ptr(),
    mime_type: c"application/x-font-os2".as_ptr(),
    group: c"font".as_ptr(),
    description: c"OS/2 Presentation Manager Bitmap Font (.fnt / .fmf)".as_ptr(),
    extensions: [c"fnt".as_ptr(), c"fmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OS2_FTCOMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"os2_ftcomp".as_ptr(),
    mime_type: c"application/x-os2-pack2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM OS/2 PACK2 FTCOMP Compressed Installation Archive (.ba_ / .ex_ / .dl_)".as_ptr(),
    extensions: [c"ba_".as_ptr(), c"ex_".as_ptr(), c"dl_".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OS2_INF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"os2_inf".as_ptr(),
    mime_type: c"application/x-os2-inf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"IBM OS/2 Presentation Manager IPF Hypertext Help / Book (.hlp / .inf)".as_ptr(),
    extensions: [c"hlp".as_ptr(), c"inf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OS2_PACK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"os2_pack".as_ptr(),
    mime_type: c"application/x-os2-pack".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM OS/2 PACK Compressed Installation File (.ex$ / .dl$ / .sy$)".as_ptr(),
    extensions: [c"ex".as_ptr(), c"dl".as_ptr(), c"sy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OSASK_HRB_EXEC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"osask_hrb_exec".as_ptr(),
    mime_type: c"application/x-osask-hrb-exec".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"OSASK / Haribote OS Compressed Application Binary (.hrb, OSASKCMP)".as_ptr(),
    extensions: [c"hrb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OSGI_FRAMEWORK_INFO4: MagikaTypeInfo = MagikaTypeInfo {
    label: c"osgi_framework_info4".as_ptr(),
    mime_type: c"application/x-osgi-framework-info4".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Eclipse OSGi Framework Bundle Info Cache (framework.info.*)".as_ptr(),
    extensions: [c"4".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OSL_OSO_BINARY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"osl_oso_binary".as_ptr(),
    mime_type: c"application/x-osl-oso-binary".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Compiled Binary OSO Shader (.oso)".as_ptr(),
    extensions: [c"oso".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"osm".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"OpenStreetMap data".as_ptr(),
    extensions: [c"osm".as_ptr(), c"pbf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"otf".as_ptr(),
    mime_type: c"font/otf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"OpenType font".as_ptr(),
    extensions: [c"otf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OUTLOOK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"outlook".as_ptr(),
    mime_type: c"application/vnd.ms-outlook".as_ptr(),
    group: c"application".as_ptr(),
    description: c"MS Outlook Message".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static OUTLOOK_NK2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"outlook_nk2".as_ptr(),
    mime_type: c"application/x-ms-outlook-nk2".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Outlook Nickname AutoComplete Cache (.nk2)".as_ptr(),
    extensions: [c"nk2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PABLOPAINT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pablopaint".as_ptr(),
    mime_type: c"image/x-pablopaint".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Atari ST PabloPaint Packed Picture (.pa3 / .ppp)".as_ptr(),
    extensions: [c"pa3".as_ptr(), c"ppp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PACK_ICE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pack_ice".as_ptr(),
    mime_type: c"application/x-pack-ice".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST Pack-Ice Compressed File (ICE! / Ice!)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PADS_PCB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pads_pcb".as_ptr(),
    mime_type: c"application/x-pads-pcb".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Mentor Graphics PADS Layout Binary PCB Design (.pcb)".as_ptr(),
    extensions: [c"pcb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PAINT4D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"paint4d".as_ptr(),
    mime_type: c"model/x-4dpaint".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"4D Paint 3D Texture Painting Project (.4dp)".as_ptr(),
    extensions: [c"4dp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PALLADIX_PCS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"palladix_pcs".as_ptr(),
    mime_type: c"audio/x-pcs".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Palladix Amiga Audio Module (.pcs)".as_ptr(),
    extensions: [c"pcs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PALMOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"palmos".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"palmos".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PANASONIC_S3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"panasonic_s3d".as_ptr(),
    mime_type: c"application/x-panasonic-s3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Panasonic ISL_PANA_3DMAP 3D Calibration Map (.s3d) Panasonic".as_ptr(),
    extensions: [c"s3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PANASONIC_UPD_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"panasonic_upd_firmware".as_ptr(),
    mime_type: c"application/x-panasonic-upd-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Panasonic / Leica Camera UPD Firmware Update Container (.bin / .quf / .lfu, 2008) Panasonic / Leica".as_ptr(),
    extensions: [c"bin".as_ptr(), c"quf".as_ptr(), c"lfu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PANGOLIN_PRR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pangolin_prr".as_ptr(),
    mime_type: c"application/x-pangolin-prr".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Pangolin Laser Systems Request Package (.prr) Pangolin Laser Systems".as_ptr(),
    extensions: [c"prr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PAQ_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"paq_archive".as_ptr(),
    mime_type: c"application/x-paq".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Matt Mahoney PAQ Context-Mixing Archive (.paq6 / .paq7 / .paq8 / .paq9a)".as_ptr(),
    extensions: [c"paq6".as_ptr(), c"paq7".as_ptr(), c"paq8".as_ptr(), c"paq9a".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARADOX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"paradox".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Paradox database".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARADOX_MESH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"paradox_mesh".as_ptr(),
    mime_type: c"model/x-paradox-mesh".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Paradox Clausewitz / Jomini Engine Binary 3D Mesh (.mesh / .anim, 2013) Paradox Interactive".as_ptr(),
    extensions: [c"mesh".as_ptr(), c"anim".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARALLELS_HDD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"parallels_hdd".as_ptr(),
    mime_type: c"application/x-parallels-hdd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Parallels Desktop Virtual Hard Disk Image (.hdd, WithoutFreeSpace)".as_ptr(),
    extensions: [c"hdd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"parchive".as_ptr(),
    mime_type: c"application/x-par2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Parchive Parity Volume Set (.par / .par2 / .par3)".as_ptr(),
    extensions: [c"par".as_ptr(), c"par2".as_ptr(), c"par3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARQUET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"parquet".as_ptr(),
    mime_type: c"application/vnd.apache.parquet".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Apache Parquet".as_ptr(),
    extensions: [c"pqt".as_ptr(), c"parquet".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PARSONS_WRK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"parsons_wrk".as_ptr(),
    mime_type: c"application/x-parsons-wrk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Parsons Technology Resource Data (.wrk)".as_ptr(),
    extensions: [c"wrk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PASCAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pascal".as_ptr(),
    mime_type: c"text/x-pascal".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Pascal source".as_ptr(),
    extensions: [c"pas".as_ptr(), c"pp".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PASSMARK_PT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"passmark_pt".as_ptr(),
    mime_type: c"application/x-passmark-pt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"PassMark PerformanceTest Baseline Results (PTRESFILE, .pt)".as_ptr(),
    extensions: [c"pt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PASSWORDSAFE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"passwordsafe".as_ptr(),
    mime_type: c"application/x-passwordsafe".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Password Safe V3 database".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PAUL_ROBOTHAM_DAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"paul_robotham_dat".as_ptr(),
    mime_type: c"audio/x-paul-robotham-dat".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Paul Robotham Amiga Music Module (.dat)".as_ptr(),
    extensions: [c"dat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PAZLIGSAW_PZL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pazligsaw_pzl".as_ptr(),
    mime_type: c"application/x-pazligsaw-pzl".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Pazligsaw Jigsaw Puzzle Package (BOX1, .pzl)".as_ptr(),
    extensions: [c"pzl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PBM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pbm".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"pbm".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PBP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pbp".as_ptr(),
    mime_type: c"application/x-psp-pbp".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Sony PlayStation Portable Executable Package (.pbp / EBOOT.PBP)".as_ptr(),
    extensions: [c"pbp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PC88_T88: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pc88_t88".as_ptr(),
    mime_type: c"application/x-pc88-t88".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"NEC PC-8801 T88 Cassette Tape Image (.t88, 1999)".as_ptr(),
    extensions: [c"t88".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcap".as_ptr(),
    mime_type: c"application/vnd.tcpdump.pcap".as_ptr(),
    group: c"application".as_ptr(),
    description: c"pcap capture file".as_ptr(),
    extensions: [c"pcap".as_ptr(), c"pcapng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCAPNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcapng".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"binary".as_ptr(),
    description: c"Packet capture PCAPNG".as_ptr(),
    extensions: [c"pcapng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCBOARD_PPE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcboard_ppe".as_ptr(),
    mime_type: c"application/x-pcboard-ppe".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Clark Development PCBoard PPL Executable (.ppe, 1993)".as_ptr(),
    extensions: [c"ppe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcd".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Point Cloud Data".as_ptr(),
    extensions: [c"pcd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcf".as_ptr(),
    mime_type: c"application/x-font-pcf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"X11 / Linux Portable Compiled Format Bitmap Font (.pcf / .pmf)".as_ptr(),
    extensions: [c"pcf".as_ptr(), c"pmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCR_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcr_font".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Keith P. Graham OPTIKS / PC-Rocks Font (.pcr)".as_ptr(),
    extensions: [c"pcr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCTOOLS_MSG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pctools_msg".as_ptr(),
    mime_type: c"application/x-pctools-msg".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Central Point PC Tools Desktop Help / Message File (.msg / .hlp)".as_ptr(),
    extensions: [c"msg".as_ptr(), c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PCX_LIBRARY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pcx_library".as_ptr(),
    mime_type: c"application/x-pcx-library".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Genus Microprogramming PCX Programmer's Toolkit Image Library (.pcl)".as_ptr(),
    extensions: [c"pcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Program Database".as_ptr(),
    extensions: [c"pdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdf".as_ptr(),
    mime_type: c"application/pdf".as_ptr(),
    group: c"document".as_ptr(),
    description: c"PDF document".as_ptr(),
    extensions: [c"pdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDFANNOTATOR_SIL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdfannotator_sil".as_ptr(),
    mime_type: c"application/x-pdfannotator-sil".as_ptr(),
    group: c"application".as_ptr(),
    description: c"GRAHL Software Design PDF Annotator Stamp/Item Library (.sil) GRAHL Software Design".as_ptr(),
    extensions: [c"sil".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDI2_DICTIONARY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdi2_dictionary".as_ptr(),
    mime_type: c"application/x-pdi2-dictionary".as_ptr(),
    group: c"database".as_ptr(),
    description: c"PDI2 Bilingual Dictionary Database (.pdi)".as_ptr(),
    extensions: [c"pdi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdn".as_ptr(),
    mime_type: c"image/x-paintnet".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Paint.NET Layered Raster Image (.pdn)".as_ptr(),
    extensions: [c"pdn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDP10_FAILSAFE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdp10_failsafe".as_ptr(),
    mime_type: c"application/x-pdp10-failsafe".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DEC PDP-10 TOPS-10 FAILSAFE Tape Stream (1970) Digital Equipment".as_ptr(),
    extensions: [c"fs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDP11_LDA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdp11_lda".as_ptr(),
    mime_type: c"application/x-pdp11-lda".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"DEC PDP-11 Absolute Loader Paper Tape Binary (.lda / .bin, 1970) Digital Equipment".as_ptr(),
    extensions: [c"lda".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDP8_SV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdp8_sv".as_ptr(),
    mime_type: c"application/x-pdp8-sv".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"DEC PDP-8 OS/8 Save Core Image Executable (.sv, 1971) Digital Equipment".as_ptr(),
    extensions: [c"sv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PDSE_PROG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pdse_prog".as_ptr(),
    mime_type: c"application/x-ibm-pdse-program".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"IBM z/OS PDSE Program Object (IEWPLMH)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PEA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pea".as_ptr(),
    mime_type: c"application/x-pea".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"PeaZip PEA archive (.pea)".as_ptr(),
    extensions: [c"pea".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PEBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pebin".as_ptr(),
    mime_type: c"application/x-dosexec".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"PE Windows executable".as_ptr(),
    extensions: [c"exe".as_ptr(), c"dll".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PEM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pem".as_ptr(),
    mime_type: c"application/x-pem-file".as_ptr(),
    group: c"application".as_ptr(),
    description: c"PEM certificate".as_ptr(),
    extensions: [c"pem".as_ptr(), c"pub".as_ptr(), c"gpg".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PEPAKURA_PDI1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pepakura_pdi1".as_ptr(),
    mime_type: c"application/x-pepakura-pdi1".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Tama Software Pepakura Designer Legacy v1 3D Papercraft Model (.pdo, 2002) Tama Software".as_ptr(),
    extensions: [c"pdo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PERF_JITDUMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"perf_jitdump".as_ptr(),
    mime_type: c"application/x-perf-jitdump".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Linux Kernel perf JIT Dump Binary Log (.dump)".as_ptr(),
    extensions: [c"dump".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PERL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"perl".as_ptr(),
    mime_type: c"text/x-perl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Perl source".as_ptr(),
    extensions: [c"pl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PERQ_SEG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"perq_seg".as_ptr(),
    mime_type: c"application/x-perq-seg".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Three Rivers PERQ POS / CMU Accent Q-Code Segment (.seg, 1980) Three Rivers Computer".as_ptr(),
    extensions: [c"seg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PFM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pfm".as_ptr(),
    mime_type: c"image/x-portable-floatmap".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Portable FloatMap HDR Raster Image (.pfm)".as_ptr(),
    extensions: [c"pfm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PFPART3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pfpart3".as_ptr(),
    mime_type: c"application/x-pfpart3".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"PFPART3 Partial Download Container (.pfpart)".as_ptr(),
    extensions: [c"pfpart".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PFR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pfr".as_ptr(),
    mime_type: c"application/font-tdpfr".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Bitstream TrueDoc Portable Font Resource (.pfr)".as_ptr(),
    extensions: [c"pfr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PGF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pgf".as_ptr(),
    mime_type: c"image/x-pgf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"libPGF Progressive Graphics File Wavelet Image (.pgf)".as_ptr(),
    extensions: [c"pgf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PGP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pgp".as_ptr(),
    mime_type: c"application/pgp-keys".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"PGP".as_ptr(),
    extensions: [c"gpg".as_ptr(), c"pgp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PGP_LOCSTR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pgp_locstr".as_ptr(),
    mime_type: c"application/x-pgp-locstr".as_ptr(),
    group: c"database".as_ptr(),
    description: c"PGP Desktop Compiled Localization String Table (.str, 2005) PGP".as_ptr(),
    extensions: [c"str".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PHARO_FUEL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pharo_fuel".as_ptr(),
    mime_type: c"application/x-pharo-fuel".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Pharo Smalltalk FUEL Object Serializer Stream (.fuel, 2011) RMoD INRIA".as_ptr(),
    extensions: [c"fuel".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PHM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"phm".as_ptr(),
    mime_type: c"image/x-portable-halfmap".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Portable HalfMap 16-Bit Float HDR Image (.phm)".as_ptr(),
    extensions: [c"phm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PHOTOSHOP_ABR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"photoshop_abr".as_ptr(),
    mime_type: c"application/x-photoshop-abr".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Adobe Photoshop Brush Preset (.abr)".as_ptr(),
    extensions: [c"abr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PHP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"php".as_ptr(),
    mime_type: c"text/x-php".as_ptr(),
    group: c"code".as_ptr(),
    description: c"PHP source".as_ptr(),
    extensions: [c"php".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PIAPRO_NTDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"piapro_ntdb".as_ptr(),
    mime_type: c"application/x-piapro-ntdb".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Crypton Future Media Piapro Studio NT Voice Database (.ntdb, 2020) Crypton Future Media".as_ptr(),
    extensions: [c"ntdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PICASA_YTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"picasa_ytf".as_ptr(),
    mime_type: c"application/x-picasa-ytf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Google Picasa Rasterized Font Metrics & Glyph Atlas (.ytf)".as_ptr(),
    extensions: [c"ytf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PICKLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pickle".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Python pickle".as_ptr(),
    extensions: [c"pickle".as_ptr(), c"pkl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PIERRE_ADANE_PAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pierre_adane_pap".as_ptr(),
    mime_type: c"audio/x-pierre-adane-pap".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Pierre Adane Packer Amiga Module (.pap)".as_ptr(),
    extensions: [c"pap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PISMO_CISO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pismo_ciso".as_ptr(),
    mime_type: c"application/x-pismo-ciso".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Pismo File Mount Compact ISO Archive (.cfs / .ciso)".as_ptr(),
    extensions: [c"cfs".as_ptr(), c"ciso".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PIXEL_PAINTERS_FMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pixel_painters_fmf".as_ptr(),
    mime_type: c"audio/x-pixel-painters-fmf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Pixel Painters AdLib Music (.fmf)".as_ptr(),
    extensions: [c"fmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PIXEL_PAINTERS_FTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pixel_painters_ftf".as_ptr(),
    mime_type: c"audio/x-pixel-painters-ftf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Pixel Painters Xatax FM Module (.ftf)".as_ptr(),
    extensions: [c"ftf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PIXINSIGHT_XISF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pixinsight_xisf".as_ptr(),
    mime_type: c"application/x-xisf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Pleiades Astrophoto PixInsight Extensible Image Serialization Format (.xisf)".as_ptr(),
    extensions: [c"xisf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PKCS15_KY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pkcs15_ky".as_ptr(),
    mime_type: c"application/x-pkcs15-ky".as_ptr(),
    group: c"application".as_ptr(),
    description: c"PKCS #15 Cryptographic Token Info (OID 1.2.840.113549.1.15.3.1, .ky)".as_ptr(),
    extensions: [c"ky".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PKM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pkm".as_ptr(),
    mime_type: c"image/x-pkm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Ericsson Texture Compression ETC1/ETC2 PKM Container (.pkm)".as_ptr(),
    extensions: [c"pkm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PLATO_LESSON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"plato_lesson".as_ptr(),
    mime_type: c"application/x-plato-lesson".as_ptr(),
    group: c"document".as_ptr(),
    description: c"PLATO / NovaNET TUTOR Lesson Source Block (.1t / .1b)".as_ptr(),
    extensions: [c"1t".as_ptr(), c"1b".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PLAYERPRO_MADG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"playerpro_madg".as_ptr(),
    mime_type: c"audio/x-mad".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Antoine Rosset Macintosh PlayerPRO Module (.madg / .madk)".as_ptr(),
    extensions: [c"madg".as_ptr(), c"madk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PLEXTALK_PROJECT_FILE_IMDN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"plextalk_project_file_imdn".as_ptr(),
    mime_type: c"application/x-plextalk-project-file-imdn".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Shinano Kenshi PLEXTALK Recording Project (.imdn, PRONOM fmt/2036)".as_ptr(),
    extensions: [c"imdn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PLM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"plm".as_ptr(),
    mime_type: c"audio/x-plm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Cubic Team / Statix DisorderTracker 2 Module (.plm)".as_ptr(),
    extensions: [c"plm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PLY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ply".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Polygon File Format".as_ptr(),
    extensions: [c"ply".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PM_PICTURE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pm_picture".as_ptr(),
    mime_type: c"image/x-pm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"PM Picture (VIEW, .pm)".as_ptr(),
    extensions: [c"pm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PMC_CRUNCHER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pmc_cruncher".as_ptr(),
    mime_type: c"application/x-powerplayer-pmc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Stephan Fuhrmann PowerPlayer Music Cruncher (.pmc)".as_ptr(),
    extensions: [c"pmc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PMD_M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pmd_m".as_ptr(),
    mime_type: c"audio/x-pmd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Masahiro Kajihara (KAJA) NEC PC-98 PMD FM Module (.m)".as_ptr(),
    extensions: [c"m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PMTILES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pmtiles".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"PMTiles".as_ptr(),
    extensions: [c"pmtiles".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"png".as_ptr(),
    mime_type: c"image/png".as_ptr(),
    group: c"image".as_ptr(),
    description: c"PNG image".as_ptr(),
    extensions: [c"png".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"po".as_ptr(),
    mime_type: c"text/gettext-translation".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Portable Object (PO) for i18n".as_ptr(),
    extensions: [c"po".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static POD5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pod5".as_ptr(),
    mime_type: c"application/x-pod5".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Oxford Nanopore POD5 Raw Signal Container (.pod5)".as_ptr(),
    extensions: [c"pod5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PODBOT_PWF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"podbot_pwf".as_ptr(),
    mime_type: c"application/x-podbot-pwf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Counter-Strike POD-Bot Waypoint File (.pwf, 2001) Count Floyd".as_ptr(),
    extensions: [c"pwf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POKEMON_GARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pokemon_garc".as_ptr(),
    mime_type: c"application/x-pokemon-garc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Game Freak Nintendo 3DS GARC Archive (.garc, 2013) Game Freak".as_ptr(),
    extensions: [c"garc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POKEMON_GFPAK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pokemon_gfpak".as_ptr(),
    mime_type: c"application/x-pokemon-gfpak".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Game Freak Nintendo Switch GFPAK Archive (.gfpak, 2018) Game Freak".as_ptr(),
    extensions: [c"gfpak".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POP_PLV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pop_plv".as_ptr(),
    mime_type: c"application/x-princed-plv".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Princed Prince of Persia Level (.plv, 2003) Enrique Calot / Princed Development Team".as_ptr(),
    extensions: [c"plv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POPCAP_PAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"popcap_pam".as_ptr(),
    mime_type: c"application/x-popcap-pam".as_ptr(),
    group: c"image".as_ptr(),
    description: c"PopCap Games PAM Sprite Animation (.pam) PopCap Games".as_ptr(),
    extensions: [c"pam".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POPSTARTER_VCD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"popstarter_vcd".as_ptr(),
    mime_type: c"application/x-popstarter-vcd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sony PlayStation POPStarter Virtual CD-ROM Image (.vcd)".as_ptr(),
    extensions: [c"vcd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PORTABLE_DATABASE_PDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"portable_database_pdb".as_ptr(),
    mime_type: c"application/x-portable-database-pdb".as_ptr(),
    group: c"document".as_ptr(),
    description: c"QuickVerse / Parsons Technology Palm OS Portable Database (.pdb, PRONOM fmt/1064)".as_ptr(),
    extensions: [c"pdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PORTABLE_FORM_FILE_PFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"portable_form_file_pff".as_ptr(),
    mime_type: c"application/x-portable-form-file-pff".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Portable Form File (.pff, PRONOM fmt/322)".as_ptr(),
    extensions: [c"pff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POSTGRES_DUMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"postgres_dump".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"PostgreSQL dump".as_ptr(),
    extensions: [c"backup".as_ptr(), c"dump".as_ptr(), c"sql".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POSTSCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"postscript".as_ptr(),
    mime_type: c"application/postscript".as_ptr(),
    group: c"document".as_ptr(),
    description: c"PostScript document".as_ptr(),
    extensions: [c"ps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POWERBUILDER_PBD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"powerbuilder_pbd".as_ptr(),
    mime_type: c"application/x-powerbuilder-pbd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sybase / Appeon PowerBuilder Dynamic Library (.pbd / .pbl, 1991) Powersoft / Sybase".as_ptr(),
    extensions: [c"pbd".as_ptr(), c"pbl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POWERBUILDER_PSR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"powerbuilder_psr".as_ptr(),
    mime_type: c"application/x-powerbuilder-psr".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Sybase / Appeon PowerBuilder Standalone DataWindow Report (.psr / .dw)".as_ptr(),
    extensions: [c"psr".as_ptr(), c"dw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POWERISO_DAA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"poweriso_daa".as_ptr(),
    mime_type: c"application/x-daa".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"PowerISO Direct-Access-Archive Disc Image (.daa)".as_ptr(),
    extensions: [c"daa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POWERPACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"powerpacker".as_ptr(),
    mime_type: c"application/x-powerpacker".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Amiga PowerPacker Compressed File (PP20)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static POWERSHELL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"powershell".as_ptr(),
    mime_type: c"application/x-powershell".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Powershell source".as_ptr(),
    extensions: [c"ps1".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static POWERVR_OBJECT_DATA_POD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"powervr_object_data_pod".as_ptr(),
    mime_type: c"application/x-powervr-object-data-pod".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Imagination Technologies PowerVR GeoPOD 3D Scene (.pod, PRONOM fmt/782)".as_ptr(),
    extensions: [c"pod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PPF_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ppf_patch".as_ptr(),
    mime_type: c"application/x-ppf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Icarus / Paradox PlayStation Patch File (.ppf)".as_ptr(),
    extensions: [c"ppf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PPM_MPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ppm_mpd".as_ptr(),
    mime_type: c"application/x-ppm-mpd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Bauer+Kirch PPM-Optimierung Data (.mpd, Bauer+Kirch Monschau)".as_ptr(),
    extensions: [c"mpd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ppt".as_ptr(),
    mime_type: c"application/vnd.ms-powerpoint".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft PowerPoint CDF document".as_ptr(),
    extensions: [c"ppt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PPTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pptx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.presentationml.presentation".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft PowerPoint 2007+ document".as_ptr(),
    extensions: [c"pptx".as_ptr(), c"pptm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PREMIERE_PRMI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"premiere_prmi".as_ptr(),
    mime_type: c"application/x-premiere-prmi".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Adobe Premiere Pro Media Intelligence Sidecar Index (.prmi)".as_ptr(),
    extensions: [c"prmi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PRESET_SAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"preset_sap".as_ptr(),
    mime_type: c"application/x-sound-particles-sap".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sound Particles / Audio Plugin Preset Data Tree (.sap / .afx)".as_ptr(),
    extensions: [c"sap".as_ptr(), c"afx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PRETRACKER_PRT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pretracker_prt".as_ptr(),
    mime_type: c"audio/x-pretracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Pink / Abyss preTracker Amiga Chiptune Module (.prt)".as_ptr(),
    extensions: [c"prt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PRINTFOX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"printfox".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"c64".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PRISTON_TALE_SMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"priston_tale_smd".as_ptr(),
    mime_type: c"model/x-priston-tale-smd".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Triglow Pictures Priston Tale 3D Model / Motion (.smd / .smb)".as_ptr(),
    extensions: [c"smd".as_ptr(), c"smb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROCMON_PMC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"procmon_pmc".as_ptr(),
    mime_type: c"application/x-procmon-pmc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Sysinternals Process Monitor Configuration (.pmc)".as_ptr(),
    extensions: [c"pmc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROJECT64_PJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"project64_pj".as_ptr(),
    mime_type: c"application/x-project64-state".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Project64 Nintendo 64 Emulator Save State (.pj)".as_ptr(),
    extensions: [c"pj".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROLOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"prolog".as_ptr(),
    mime_type: c"text/x-prolog".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Prolog source".as_ptr(),
    extensions: [c"pl".as_ptr(), c"pro".as_ptr(), c"P".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PROSHAPE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proshape".as_ptr(),
    mime_type: c"image/x-proshape".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ProShape CAD Drawing (.psp)".as_ptr(),
    extensions: [c"psp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROTEINDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proteindb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Protein DB".as_ptr(),
    extensions: [c"pdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PROTEUS_ISIS_DSN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proteus_isis_dsn".as_ptr(),
    mime_type: c"application/x-proteus-isis-dsn".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Labcenter Proteus ISIS Circuit Schematic (.dsn, 1990) Labcenter Electronics".as_ptr(),
    extensions: [c"dsn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROTEUS_PDSPART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proteus_pdspart".as_ptr(),
    mime_type: c"application/x-proteus-pdspart".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Labcenter Proteus CAD Part Descriptor (.pdspart) Labcenter Electronics".as_ptr(),
    extensions: [c"pdspart".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROTO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proto".as_ptr(),
    mime_type: c"text/x-proto".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Protocol buffer definition".as_ptr(),
    extensions: [c"proto".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PROTOBUF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"protobuf".as_ptr(),
    mime_type: c"application/protobuf".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Protocol buffers".as_ptr(),
    extensions: [c"protobuf".as_ptr(), c"pb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROTREKKR_PTK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"protrekkr_ptk".as_ptr(),
    mime_type: c"audio/x-protrekkr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Franck Charlet ProTrekkr / NoiseTrekkr Module (.ptk, PROTREKK / TWNN)".as_ptr(),
    extensions: [c"ptk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROX_USB_XHA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"prox_usb_xha".as_ptr(),
    mime_type: c"application/x-prox-usb-xha".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Pro-face PROX_USB_XS HMI Project Archive (.xha) Pro-face / Schneider Electric".as_ptr(),
    extensions: [c"xha".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PROXMOX_PBS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"proxmox_pbs".as_ptr(),
    mime_type: c"application/x-proxmox-backup".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Proxmox Backup Server Indexed Archive / Blob (.pxar / .fidx / .didx / .blob)".as_ptr(),
    extensions: [c"pxar".as_ptr(), c"fidx".as_ptr(), c"didx".as_ptr(), c"blob".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PRVW_PRW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"prvw_prw".as_ptr(),
    mime_type: c"application/x-prvw-prw".as_ptr(),
    group: c"image".as_ptr(),
    description: c"PRVW Preview Image Container (.prw)".as_ptr(),
    extensions: [c"prw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS1_MCR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps1_mcr".as_ptr(),
    mime_type: c"application/x-ps1-memorycard".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sony PlayStation 1 Memory Card Image (.mcr / .mcd)".as_ptr(),
    extensions: [c"mcr".as_ptr(), c"mcd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS2_HD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps2_hd".as_ptr(),
    mime_type: c"audio/x-ps2-hd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony PlayStation 2 SCEI SoundBank Header (.hd)".as_ptr(),
    extensions: [c"hd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS2_IPU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps2_ipu".as_ptr(),
    mime_type: c"video/x-ps2-ipu".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Sony PlayStation 2 IPU MPEG-2 Intra-Frame Video Stream (.ipu, ipum)".as_ptr(),
    extensions: [c"ipu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS2_VMC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps2_vmc".as_ptr(),
    mime_type: c"application/x-ps2-memorycard".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sony PlayStation 2 Memory Card Image (.ps2 / .vmc)".as_ptr(),
    extensions: [c"ps2".as_ptr(), c"vmc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS3_PKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps3_pkg".as_ptr(),
    mime_type: c"application/x-ps3-pkg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sony PlayStation 3 / PSP Package (.pkg)".as_ptr(),
    extensions: [c"pkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS4_PKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps4_pkg".as_ptr(),
    mime_type: c"application/x-ps4-pkg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sony PlayStation 4 / PS Vita Content Package (.pkg)".as_ptr(),
    extensions: [c"pkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PS_SFO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ps_sfo".as_ptr(),
    mime_type: c"application/x-sony-sfo".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sony PlayStation System File Object (PARAM.SFO / .sfo)".as_ptr(),
    extensions: [c"sfo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSA_MODULE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psa_module".as_ptr(),
    mime_type: c"audio/x-psa".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Professional Sound Artists Amiga Module (.psa)".as_ptr(),
    extensions: [c"psa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psc".as_ptr(),
    mime_type: c"audio/x-psc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Pro Sound Creator Module (.psc)".as_ptr(),
    extensions: [c"psc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psd".as_ptr(),
    mime_type: c"image/vnd.adobe.photoshop".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Adobe Photoshop".as_ptr(),
    extensions: [c"psd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psf".as_ptr(),
    mime_type: c"image/x-arcsoft-psf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ArcSoft PhotoStudio Image (.psf, PRONOM fmt/1832)".as_ptr(),
    extensions: [c"psf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSG_SLEEP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psg_sleep".as_ptr(),
    mime_type: c"application/x-psg-sleep".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Polysomnography (PSG) Sleep Study Recording (.psg)".as_ptr(),
    extensions: [c"psg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSION3_WORD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psion3_word".as_ptr(),
    mime_type: c"application/x-psion3-word".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Psion Series 3 Word Document (.wrd)".as_ptr(),
    extensions: [c"wrd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSION_OPL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psion_opl".as_ptr(),
    mime_type: c"application/x-psion-opl".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Psion Organiser II / Series 3 OPL Translated Bytecode (.opo / .opa)".as_ptr(),
    extensions: [c"opo".as_ptr(), c"opa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSION_SIBO_APP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psion_sibo_app".as_ptr(),
    mime_type: c"application/x-psion-sibo-app".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Psion Series 3 SIBO IMG / APP / DYL Executable (.img / .app / .dyl)".as_ptr(),
    extensions: [c"img".as_ptr(), c"app".as_ptr(), c"dyl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSM_MASI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psm_masi".as_ptr(),
    mime_type: c"audio/x-psm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Epic MegaGames MASI ProTracker Studio Module (.psm)".as_ptr(),
    extensions: [c"psm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSP_DAX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psp_dax".as_ptr(),
    mime_type: c"application/x-psp-dax".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Dark_AleX PlayStation Portable DAX Compressed ISO (.dax)".as_ptr(),
    extensions: [c"dax".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSX_EXE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psx_exe".as_ptr(),
    mime_type: c"application/x-psx-executable".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Sony PlayStation 1 PS-X EXE Executable".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PSY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"psy".as_ptr(),
    mime_type: c"audio/x-psy".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Psycle Modular Music Creation Studio Song (.psy)".as_ptr(),
    extensions: [c"psy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PTEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ptex".as_ptr(),
    mime_type: c"image/x-ptex".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Walt Disney Animation Studios Ptex Per-Face Texture (.ptx)".as_ptr(),
    extensions: [c"ptx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ptf".as_ptr(),
    mime_type: c"application/x-protools-session".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Digidesign / Avid Pro Tools Session (.ptf / .pts)".as_ptr(),
    extensions: [c"ptf".as_ptr(), c"pts".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ptm".as_ptr(),
    mime_type: c"audio/x-ptm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Lone Ranger / Acme PolyTracker Module (.ptm)".as_ptr(),
    extensions: [c"ptm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PUB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pub".as_ptr(),
    mime_type: c"application/x-mspublisher".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"pub".as_ptr(),
    extensions: [c"pub".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PUFFIN_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"puffin_patch".as_ptr(),
    mime_type: c"application/x-puffin-patch".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Google ChromeOS / Android Puffin Deterministic Deflate Patch (PUF1)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PUMATRACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pumatracker".as_ptr(),
    mime_type: c"audio/x-pumatracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Agile Pumatracker Amiga Module (.puma)".as_ptr(),
    extensions: [c"puma".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PVI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pvi".as_ptr(),
    mime_type: c"audio/x-pvi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NEC PC-98 FMP ADPCM Sample Bank (.pvi)".as_ptr(),
    extensions: [c"pvi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PVM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pvm".as_ptr(),
    mime_type: c"application/x-sega-pvm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sega Dreamcast PVM Texture Archive (PVMH)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PVR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pvr".as_ptr(),
    mime_type: c"image/x-pvr".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Imagination Technologies PowerVR Texture (.pvr)".as_ptr(),
    extensions: [c"pvr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PXHG_PACKAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pxhg_package".as_ptr(),
    mime_type: c"application/x-pxhg-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"PXHG Multilingual String Package (.hg)".as_ptr(),
    extensions: [c"hg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PXTONE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pxtone".as_ptr(),
    mime_type: c"audio/x-pxtone".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Studio Pixel PxTone / Piston Collage Noise or Voice (.ptnoise / .ptvoice)".as_ptr(),
    extensions: [c"ptnoise".as_ptr(), c"ptvoice".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PYTHON: MagikaTypeInfo = MagikaTypeInfo {
    label: c"python".as_ptr(),
    mime_type: c"text/x-python".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Python source".as_ptr(),
    extensions: [c"py".as_ptr(), c"pyi".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static PYTHONBYTECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pythonbytecode".as_ptr(),
    mime_type: c"application/x-bytecode.python".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Python compiled bytecode".as_ptr(),
    extensions: [c"pyc".as_ptr(), c"pyo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PYTHONPAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pythonpar".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"pythonpar".as_ptr(),
    extensions: [c"par".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PYTORCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pytorch".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Pytorch storage file".as_ptr(),
    extensions: [c"pt".as_ptr(), c"pth".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static PYTORCH_EXPORT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"pytorch_export".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"PyTorch exported program".as_ptr(),
    extensions: [c"pt2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static Q_A_WORD_PROCESSOR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"q_a_word_processor".as_ptr(),
    mime_type: c"application/x-q-a-word-processor".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Symantec Q&A Word Processor Document (.jw, PRONOM fmt/1045)".as_ptr(),
    extensions: [c"jw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QCA_ATH_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qca_ath_firmware".as_ptr(),
    mime_type: c"application/x-qca-ath-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Atheros WLAN / WiGig Firmware & Board Data (QCA-ATH10K/11K/12K/6KL)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QCA_BT_TLV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qca_bt_tlv".as_ptr(),
    mime_type: c"application/x-qca-bluetooth-tlv".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Atheros Bluetooth Rampatch & NVM TLV Firmware (.bin / .tlv)".as_ptr(),
    extensions: [c"bin".as_ptr(), c"tlv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QCDT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qcdt".as_ptr(),
    mime_type: c"application/x-qualcomm-qcdt".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Qualcomm Bootloader Device Tree Table Image (QCDT)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QCOW2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qcow2".as_ptr(),
    mime_type: c"application/x-raw-disk-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"QEMU Copy-On-Write Virtual Disk Image (.qcow2)".as_ptr(),
    extensions: [c"qcow2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QEMU_QED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qemu_qed".as_ptr(),
    mime_type: c"application/x-qemu-qed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"QEMU Enhanced Disk Image (.qed, QED\\0)".as_ptr(),
    extensions: [c"qed".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QGIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qgis".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"QGIS project archive".as_ptr(),
    extensions: [c"qgz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QNX_DEFLATE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qnx_deflate".as_ptr(),
    mime_type: c"application/x-qnx-deflate".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"BlackBerry QNX deflate Block-Compressed Filesystem Stream (.deflated)".as_ptr(),
    extensions: [c"deflated".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QNX_IFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qnx_ifs".as_ptr(),
    mime_type: c"application/x-qnx-ifs".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BlackBerry QNX Neutrino Image Filesystem Boot Image (.ifs)".as_ptr(),
    extensions: [c"ifs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QOI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qoi".as_ptr(),
    mime_type: c"image/x-qoi".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Quite Ok Image".as_ptr(),
    extensions: [c"qoi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qt".as_ptr(),
    mime_type: c"video/quicktime".as_ptr(),
    group: c"video".as_ptr(),
    description: c"QuickTime".as_ptr(),
    extensions: [c"mov".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUAKE3_QVM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quake3_qvm".as_ptr(),
    mime_type: c"application/x-quake3-qvm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"id Tech 3 Quake III Arena Virtual Machine Bytecode (.qvm, 1999) id Software".as_ptr(),
    extensions: [c"qvm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUAKE_MDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quake_mdl".as_ptr(),
    mime_type: c"model/x-quake-mdl".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"id Software Quake 1 3D Model (.mdl)".as_ptr(),
    extensions: [c"mdl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUALCOMM_ACDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qualcomm_acdb".as_ptr(),
    mime_type: c"application/x-qualcomm-acdb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Audio Calibration Database (.acdb)".as_ptr(),
    extensions: [c"acdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUALCOMM_FBPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qualcomm_fbpk".as_ptr(),
    mime_type: c"application/x-qualcomm-fbpk".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Fastboot Firmware Pack (FBPK)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUALCOMM_META_IMAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qualcomm_meta_image".as_ptr(),
    mime_type: c"application/x-qualcomm-meta-image".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Android Bootloader Meta Image (meta_image)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUALCOMM_MIBIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qualcomm_mibib".as_ptr(),
    mime_type: c"application/x-qualcomm-mibib".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm MSM MIBIB Flash Partition Table (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUALCOMM_SBL1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qualcomm_sbl1".as_ptr(),
    mime_type: c"application/x-qualcomm-sbl1".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Secondary Bootloader SBL1 / NorCodeWord Image (.mbn)".as_ptr(),
    extensions: [c"mbn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUALCOMM_SPLASH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"qualcomm_splash".as_ptr(),
    mime_type: c"application/x-qualcomm-splash".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Qualcomm Snapdragon Boot Splash Screen Image (.img)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUARTET_SET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quartet_set".as_ptr(),
    mime_type: c"audio/x-quartet-set".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MicroDeal Atari ST Quartet Sample Set (.set)".as_ptr(),
    extensions: [c"set".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUARTET_ST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quartet_st".as_ptr(),
    mime_type: c"audio/x-quartet".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MicroDeal Atari ST Quartet 4-Voice Module (.4v / .qpa)".as_ptr(),
    extensions: [c"4v".as_ptr(), c"qpa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUARTUS_POF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quartus_pof".as_ptr(),
    mime_type: c"application/x-altera-pof".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Altera / Intel Quartus Programmer Object File (.pof, POF\\\\0)".as_ptr(),
    extensions: [c"pof".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUATTRO_PRO_WB1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quattro_pro_wb1".as_ptr(),
    mime_type: c"application/x-quattro-pro-wb1".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Borland Quattro Pro for Windows Encrypted/Packed Notebook (.wb1) Borland".as_ptr(),
    extensions: [c"wb1".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUEST_CAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quest_cas".as_ptr(),
    mime_type: c"application/x-quest-cas".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Quest 3/4 Compiled Game (.cas)".as_ptr(),
    extensions: [c"cas".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUICKEN_3_DATABASE_FILE_QST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quicken_3_database_file_qst".as_ptr(),
    mime_type: c"application/x-quicken-3-database-file-qst".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Intuit Quicken 3 Database File (.qst, PRONOM fmt/1861)".as_ptr(),
    extensions: [c"qst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static QUICKLINK_QFX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"quicklink_qfx".as_ptr(),
    mime_type: c"image/x-quicklink-fax".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Smith Micro QuickLink II Fax Image (.qfx)".as_ptr(),
    extensions: [c"qfx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static R: MagikaTypeInfo = MagikaTypeInfo {
    label: c"r".as_ptr(),
    mime_type: c"text/x-R".as_ptr(),
    group: c"code".as_ptr(),
    description: c"R (language)".as_ptr(),
    extensions: [c"R".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RACER_DOF1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"racer_dof1".as_ptr(),
    mime_type: c"application/x-racer-dof1".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Racer Simulation Dynamic Object Format 3D Model (.dof, 2001) Ruud van Gaal".as_ptr(),
    extensions: [c"dof".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAD_BINK_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rad_bink_audio".as_ptr(),
    mime_type: c"audio/x-rad-bink-audio".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"RAD Game Tools Bink Audio Stream (.binka)".as_ptr(),
    extensions: [c"binka".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RADDBG_RDI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"raddbg_rdi".as_ptr(),
    mime_type: c"application/x-raddbg-rdi".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"RAD Debugger Debug Information (.rdi)".as_ptr(),
    extensions: [c"rdi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RADICAL_RCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"radical_rcf".as_ptr(),
    mime_type: c"application/x-radical-cement-library".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Radical Entertainment Cement Library Archive (.rcf)".as_ptr(),
    extensions: [c"rcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RADICAL_RSD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"radical_rsd".as_ptr(),
    mime_type: c"audio/x-radical-rsd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Radical Entertainment RSD Audio Stream (.rsd)".as_ptr(),
    extensions: [c"rsd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RADIOCEF_RCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"radiocef_rcf".as_ptr(),
    mime_type: c"application/x-radiocef-rcf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Radiocef Studio Dental X-Ray Cephalometry (.rcf)".as_ptr(),
    extensions: [c"rcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAGE_AWC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rage_awc".as_ptr(),
    mime_type: c"audio/x-rage-awc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Rockstar Games RAGE Audio Wave Container (.awc)".as_ptr(),
    extensions: [c"awc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAGE_RSC7: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rage_rsc7".as_ptr(),
    mime_type: c"application/x-rage-rsc7".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Rockstar Games RAGE Engine Resource Container v7 (.ydr / .ytd / .yft / .ydd / .ymap / .ytyp / .ybn / .ycd)".as_ptr(),
    extensions: [c"ydr".as_ptr(), c"ytd".as_ptr(), c"yft".as_ptr(), c"ydd".as_ptr(), c"ymap".as_ptr(), c"ytyp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAGTIME: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ragtime".as_ptr(),
    mime_type: c"application/x-ragtime".as_ptr(),
    group: c"document".as_ptr(),
    description: c"RagTime Business Publishing Document (.rag / .rtd)".as_ptr(),
    extensions: [c"rag".as_ptr(), c"rtd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAILWORKS_DAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"railworks_dav".as_ptr(),
    mime_type: c"audio/x-railworks-dav".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"RailWorks / Train Simulator XOR-Obfuscated WAV Audio (.dav)".as_ptr(),
    extensions: [c"dav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAILWORKS_SERZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"railworks_serz".as_ptr(),
    mime_type: c"application/x-railworks-serz".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Kuju Entertainment RailWorks / Train Simulator SERZ Binary XML (.tgpcdx / .geopcdx / .bin)".as_ptr(),
    extensions: [c"tgpcdx".as_ptr(), c"geopcdx".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAMTRACKER_TRK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ramtracker_trk".as_ptr(),
    mime_type: c"audio/x-trk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"RamTracker Module (.trk)".as_ptr(),
    extensions: [c"trk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAN_ONLINE_IMS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ran_online_ims".as_ptr(),
    mime_type: c"application/x-ran-online-ims".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Min Communications Ran Online GLITEM_MIX Item Crafting Table (.ims)".as_ptr(),
    extensions: [c"ims".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RANDOMBYTES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"randombytes".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Random bytes".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RANDOMTXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"randomtxt".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Random text".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RAPISCAN_RCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rapiscan_rcf".as_ptr(),
    mime_type: c"application/x-rapiscan-rcf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Rapiscan Dual-Energy X-Ray Scan Archive (.rcf, 0xFFFF1111)".as_ptr(),
    extensions: [c"rcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rar".as_ptr(),
    mime_type: c"application/vnd.rar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"RAR archive data".as_ptr(),
    extensions: [c"rar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rarc".as_ptr(),
    mime_type: c"application/x-nintendo-rarc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo GameCube / Wii JKR Archive (.rarc / .arc)".as_ptr(),
    extensions: [c"rarc".as_ptr(), c"arc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RAVEN_XBNF_BNX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"raven_xbnf_bnx".as_ptr(),
    mime_type: c"application/x-raven-xbnf".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Raven Software Engine XBNF Binary Font / Text Resource (.bnx)".as_ptr(),
    extensions: [c"bnx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RCBOT_RCW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rcbot_rcw".as_ptr(),
    mime_type: c"application/x-rcbot-rcw".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Half-Life RCBot Waypoint File (.rcw, 2004) Cheeseh".as_ptr(),
    extensions: [c"rcw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RDATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rdata".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"R serialized data".as_ptr(),
    extensions: [c"rda".as_ptr(), c"rdata".as_ptr(), c"rds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rdf".as_ptr(),
    mime_type: c"application/rdf+xml".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Resource Description Framework document (RDF)".as_ptr(),
    extensions: [c"rdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RDP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rdp".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"rdp".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RE4_MDT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"re4_mdt".as_ptr(),
    mime_type: c"application/x-re4-mdt".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Capcom Resident Evil 4 Event Message Table (.mdt)".as_ptr(),
    extensions: [c"mdt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RE_ENGINE_CHAIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"re_engine_chain".as_ptr(),
    mime_type: c"application/x-re-engine-chain".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Capcom RE Engine Physics Chain Descriptor (.chain.*)".as_ptr(),
    extensions: [c"chain".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RE_ENGINE_MCAMLIST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"re_engine_mcamlist".as_ptr(),
    mime_type: c"application/x-re-engine-mcamlist".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Capcom RE Engine Motion Camera List (clst, .mcamlist.*)".as_ptr(),
    extensions: [c"mcamlist".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RE_ENGINE_MDF2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"re_engine_mdf2".as_ptr(),
    mime_type: c"application/x-re-engine-mdf2".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Capcom RE Engine Material Definition v2 (.mdf2)".as_ptr(),
    extensions: [c"mdf2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALBASIC_RBBF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realbasic_rbbf".as_ptr(),
    mime_type: c"application/x-realbasic-rbbf".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"REALbasic / Xojo Binary Project or Object (RbBF, .rbp / .rbo)".as_ptr(),
    extensions: [c"rbp".as_ptr(), c"rbo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realm".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Realm database".as_ptr(),
    extensions: [c"realm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALMEDIA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realmedia".as_ptr(),
    mime_type: c"application/vnd.rn-realmedia".as_ptr(),
    group: c"video".as_ptr(),
    description: c"RealNetworks RealMedia / RealAudio Stream (.rm / .rmvb / .ra)".as_ptr(),
    extensions: [c"rm".as_ptr(), c"rmvb".as_ptr(), c"ra".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALTEK_BT_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realtek_bt_fw".as_ptr(),
    mime_type: c"application/x-realtek-bt-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Realtek Bluetooth Controller Firmware Patch & Config (Realtech)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALTEK_CSYS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realtek_csys".as_ptr(),
    mime_type: c"application/x-realtek-csys".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Realtek AP / Router CSYS Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALTEK_RTK0: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realtek_rtk0".as_ptr(),
    mime_type: c"application/x-realtek-rtk0".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Realtek RTK0 Bluetooth / Wi-Fi SoC Firmware Patch (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REALTEK_WIFI_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"realtek_wifi_fw".as_ptr(),
    mime_type: c"application/x-realtek-wifi-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Realtek IEEE 802.11 Wireless MAC/BB/RF Firmware (rtlwifi / rtw88 / rtw89)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REBELLION_ASURA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rebellion_asura".as_ptr(),
    mime_type: c"application/x-rebellion-asura".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Rebellion Developments Asura Engine Resource Archive (.asr / .pc, 2005) Rebellion Developments".as_ptr(),
    extensions: [c"asr".as_ptr(), c"pc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RECOMPOSER_RCP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"recomposer_rcp".as_ptr(),
    mime_type: c"audio/x-recomposer".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Come On Music Recomposer PC-98 / X68000 MIDI Sequencer (.rcp / .r36)".as_ptr(),
    extensions: [c"rcp".as_ptr(), c"r36".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"redb".as_ptr(),
    mime_type: c"application/x-redb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Christopher Berner redb Embedded Rust Key-Value Database (.redb / .vectre)".as_ptr(),
    extensions: [c"redb".as_ptr(), c"vectre".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REDCODE_R3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"redcode_r3d".as_ptr(),
    mime_type: c"video/x-red-r3d".as_ptr(),
    group: c"video".as_ptr(),
    description: c"RED Digital Cinema REDCODE RAW Video (.r3d)".as_ptr(),
    extensions: [c"r3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REDENGINE_RDAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"redengine_rdar".as_ptr(),
    mime_type: c"application/x-redengine-rdar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CD Projekt RED REDengine 4 Archive (.archive, 2020) CD Projekt RED".as_ptr(),
    extensions: [c"archive".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REDIS_AOF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"redis_aof".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Redis append-only log".as_ptr(),
    extensions: [c"aof".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REDIS_RDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"redis_rdb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Redis snapshot".as_ptr(),
    extensions: [c"rdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REDTHUMB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"redthumb".as_ptr(),
    mime_type: c"image/x-red-thumbnail".as_ptr(),
    group: c"image".as_ptr(),
    description: c"RED Digital Cinema Thumbnail (.rtn, PRONOM fmt/1664)".as_ptr(),
    extensions: [c"rtn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REFPACK_BIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"refpack_big".as_ptr(),
    mime_type: c"application/x-refpack-big".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Electronic Arts RefPack-Compressed BIGF Archive (.big, Electronic Arts)".as_ptr(),
    extensions: [c"big".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static REKO_CARDSET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"reko_cardset".as_ptr(),
    mime_type: c"image/x-reko-cardset".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Amiga / PC REKO Klondike Cardset (.reko)".as_ptr(),
    extensions: [c"reko".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RELIC_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"relic_archive".as_ptr(),
    mime_type: c"application/x-relic-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Relic Entertainment SGA / Big Archive (.sga / .big, 2003) Relic Entertainment".as_ptr(),
    extensions: [c"sga".as_ptr(), c"big".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RELIC_RBH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"relic_rbh".as_ptr(),
    mime_type: c"application/x-relic-rbh".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Relic Entertainment PIFF RBHF RigidBody Header (.rbh, 2004) Relic Entertainment".as_ptr(),
    extensions: [c"rbh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RESCSEG_CMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rescseg_cmf".as_ptr(),
    mime_type: c"application/x-rescseg-cmf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"RESCSEG Segmented Resource Package (.cmf)".as_ptr(),
    extensions: [c"cmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RESFILE_RSC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"resfile_rsc".as_ptr(),
    mime_type: c"application/x-resfile-rsc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ResFile Zlib-Compressed Resource Container (.rsc)".as_ptr(),
    extensions: [c"rsc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RFZPACK_RFZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rfzpack_rfz".as_ptr(),
    mime_type: c"application/x-rfzpack-rfz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"RFZPACK Resource Archive (.rfz)".as_ptr(),
    extensions: [c"rfz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RGMH_SAVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rgmh_save".as_ptr(),
    mime_type: c"application/x-windows-rgmh-save".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Game Explorer Rich Saved Game (.sav / .pcsav)".as_ptr(),
    extensions: [c"sav".as_ptr(), c"pcsav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RGSSAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rgssad".as_ptr(),
    mime_type: c"application/x-rgss-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Enterbrain RPG Maker XP / VX / VX Ace RGSS Archive (.rgssad / .rgss2a / .rgss3a)".as_ptr(),
    extensions: [c"rgssad".as_ptr(), c"rgss2a".as_ptr(), c"rgss3a".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RHIDE_GDT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rhide_gdt".as_ptr(),
    mime_type: c"application/x-rhide-gdt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Robert Hoehne RHIDE DJGPP IDE Project / Desktop File (.gpr / .gdt, 1996) Robert Hoehne".as_ptr(),
    extensions: [c"gpr".as_ptr(), c"gdt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RHINOCEROS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rhinoceros".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Rhino 3DM".as_ptr(),
    extensions: [c"3dm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RIEGELI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"riegeli".as_ptr(),
    mime_type: c"application/x-riegeli".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Google Riegeli Records Structured Container (.riegeli)".as_ptr(),
    extensions: [c"riegeli".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RILLSOFT_RPJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rillsoft_rpj".as_ptr(),
    mime_type: c"application/x-rillsoft-rpj".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Rillsoft Project Management Schedule (.rpj / .mbk) Rillsoft".as_ptr(),
    extensions: [c"rpj".as_ptr(), c"mbk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RISCOS_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"riscos_font".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Acorn RISC OS Outline / IntMetrics Font (Outlines / IntMetrics)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RIVE_RIV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rive_riv".as_ptr(),
    mime_type: c"application/x-rive".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Rive Interactive Vector Animation (.riv)".as_ptr(),
    extensions: [c"riv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RJP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rjp".as_ptr(),
    mime_type: c"audio/x-rjp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Richard Joseph / Vectordean Amiga Audio (.rjp / .sng / .ins)".as_ptr(),
    extensions: [c"rjp".as_ptr(), c"sng".as_ptr(), c"ins".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RKA_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rka_audio".as_ptr(),
    mime_type: c"audio/x-rka".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Malcolm Taylor RK Audio Lossless Compressor (.rka, RKA7)".as_ptr(),
    extensions: [c"rka".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RLI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rli".as_ptr(),
    mime_type: c"image/x-rli".as_ptr(),
    group: c"image".as_ptr(),
    description: c"RealWorld Paint Layered Image (.rli)".as_ptr(),
    extensions: [c"rli".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RLIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rlib".as_ptr(),
    mime_type: c"application/x-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"rust library".as_ptr(),
    extensions: [c"rlib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RNC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rnc".as_ptr(),
    mime_type: c"application/x-rnc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Rob Northen Computing ProPack Compressed File (RNC)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RNS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rns".as_ptr(),
    mime_type: c"audio/x-rns".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Renoise Legacy Binary Song (.rns)".as_ptr(),
    extensions: [c"rns".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROB_HUBBARD_RHO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rob_hubbard_rho".as_ptr(),
    mime_type: c"audio/x-rob-hubbard-st".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Rob Hubbard Atari ST Music Module (.rho)".as_ptr(),
    extensions: [c"rho".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROBOPLAY_PLY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"roboplay_ply".as_ptr(),
    mime_type: c"application/x-roboplay-ply".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ROBOPLAY AdLib OPL2 Chiptune Module (.ply)".as_ptr(),
    extensions: [c"ply".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROCKCHIP_RKAF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rockchip_rkaf".as_ptr(),
    mime_type: c"application/x-rockchip-rkaf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Rockchip RKAF Update Archive Image (.img)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROCKCHIP_RKFW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rockchip_rkfw".as_ptr(),
    mime_type: c"application/x-rockchip-rkfw".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Rockchip RKFW Bootable Firmware Package (.img)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROCKCHIP_RSCE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rockchip_rsce".as_ptr(),
    mime_type: c"application/x-rockchip-rsce".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Rockchip RSCE Android Resource Image (.img)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROCKSDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rocksdb".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"RocksDB".as_ptr(),
    extensions: [c"sst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROCKSTAR_RPF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rockstar_rpf".as_ptr(),
    mime_type: c"application/x-rockstar-rpf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Rockstar Games RAGE Package File Archive (.rpf, 2008) Rockstar Games".as_ptr(),
    extensions: [c"rpf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROCKY_RIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rocky_rip".as_ptr(),
    mime_type: c"image/x-atari-rip".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Atari 8-bit Rocky Interlace Picture (.rip)".as_ptr(),
    extensions: [c"rip".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROLAND_SVQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"roland_svq".as_ptr(),
    mime_type: c"audio/x-svq".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Roland XP/JV MRC-Pro Sequencer Song (.svq)".as_ptr(),
    extensions: [c"svq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROMFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"romfs".as_ptr(),
    mime_type: c"application/x-romfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Linux ROMFS Read-Only Filesystem Image (.romfs)".as_ptr(),
    extensions: [c"romfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROS_BAG_FORMAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ros_bag_format".as_ptr(),
    mime_type: c"application/x-ros-bag-format".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Robot Operating System (ROS) 1.x Message Log Bag (.bag, PRONOM fmt/2093)".as_ptr(),
    extensions: [c"bag".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ROUTER_WLK_CONFIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"router_wlk_config".as_ptr(),
    mime_type: c"application/x-router-wlk-config".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TP-Link / Realtek WLK Encrypted Router Configuration (.bin) TP-Link / Realtek".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RPA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpa".as_ptr(),
    mime_type: c"application/x-renpy-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ren'Py Visual Novel Archive (.rpa)".as_ptr(),
    extensions: [c"rpa".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RPG_WOLF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpg_wolf".as_ptr(),
    mime_type: c"application/x-rpg-wolf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"WOLF RPG Editor DXLib Encrypted Archive (.wolf, 2008) SmokingWOLF".as_ptr(),
    extensions: [c"wolf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RPGMAKER_XYZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpgmaker_xyz".as_ptr(),
    mime_type: c"image/x-rpgmaker-xyz".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ASCII / Enterbrain RPG Maker 2000 / 2003 XYZ Compressed Raster Image (.xyz)".as_ptr(),
    extensions: [c"xyz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RPGMV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpgmv".as_ptr(),
    mime_type: c"application/x-rpgmaker-encrypted".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Kadokawa RPG Maker MV / MZ Encrypted Asset (.rpgmvp / .rpgmvo / .rpgmvm / .png_ / .ogg_)".as_ptr(),
    extensions: [c"rpgmvp".as_ptr(), c"rpgmvo".as_ptr(), c"rpgmvm".as_ptr(), c"png_".as_ptr(), c"ogg_".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RPM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpm".as_ptr(),
    mime_type: c"application/x-rpm".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"RedHat Package Manager archive (RPM)".as_ptr(),
    extensions: [c"rpm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RPYC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rpyc".as_ptr(),
    mime_type: c"application/x-renpy-bytecode".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Ren'Py Compiled Script AST Bytecode (.rpyc)".as_ptr(),
    extensions: [c"rpyc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rst".as_ptr(),
    mime_type: c"text/x-rst".as_ptr(),
    group: c"text".as_ptr(),
    description: c"ReStructuredText document".as_ptr(),
    extensions: [c"rst".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rtf".as_ptr(),
    mime_type: c"text/rtf".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Rich Text Format document".as_ptr(),
    extensions: [c"rtf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RTM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rtm".as_ptr(),
    mime_type: c"audio/x-rtm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"RealTech Real Tracker Module (.rtm)".as_ptr(),
    extensions: [c"rtm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RTSTRIKE_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rtstrike_font".as_ptr(),
    mime_type: c"application/x-rtstrike-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"RTSTRK01 Zlib-Compressed Font Strike (.rtstrike)".as_ptr(),
    extensions: [c"rtstrike".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RUBY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ruby".as_ptr(),
    mime_type: c"application/x-ruby".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Ruby source".as_ptr(),
    extensions: [c"rb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RUBY_RDOC_RI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ruby_rdoc_ri".as_ptr(),
    mime_type: c"application/x-ruby-rdoc-ri".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Ruby RDoc RI Marshal Documentation Entry (.ri, 2003)".as_ptr(),
    extensions: [c"ri".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RUBY_YARB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ruby_yarb".as_ptr(),
    mime_type: c"application/x-ruby-yarb".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"CRuby YARV Compiled Instruction Sequence Bytecode (YARB)".as_ptr(),
    extensions: [c"yarb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RUP_NINJA2_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rup_ninja2_patch".as_ptr(),
    mime_type: c"application/x-rup-patch".as_ptr(),
    group: c"application".as_ptr(),
    description: c"errick NINJA 2.0 Raw Ultima ROM Patch (.rup, NINJA2)".as_ptr(),
    extensions: [c"rup".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static RUST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rust".as_ptr(),
    mime_type: c"application/x-rust".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Rust source".as_ptr(),
    extensions: [c"rs".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static RZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"rzip".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Rzip".as_ptr(),
    extensions: [c"rz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static S3M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"s3m".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Scream Tracker 3 Module Audio (.s3m)".as_ptr(),
    extensions: [c"s3m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static S98: MagikaTypeInfo = MagikaTypeInfo {
    label: c"s98".as_ptr(),
    mime_type: c"audio/x-s98".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NEC PC-88 / PC-98 S98 FM Sound Chip Register Log (.s98)".as_ptr(),
    extensions: [c"s98".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SABDU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sabdu".as_ptr(),
    mime_type: c"application/x-sabdu".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SAB Diskette Utility Floppy Image (.sdu)".as_ptr(),
    extensions: [c"sdu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAFARI_BINARYCOOKIES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"safari_binarycookies".as_ptr(),
    mime_type: c"application/x-apple-binarycookies".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Apple Safari Binary Cookies Store (.binarycookies)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAFETENSORS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"safetensors".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"model".as_ptr(),
    description: c"Safetensors".as_ptr(),
    extensions: [c"safetensors".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAKURA_WARS_SBX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sakura_wars_sbx".as_ptr(),
    mime_type: c"application/x-sakura-wars-sbx".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"SEGA / Overworks Sakura Wars 3 Compiled Script (ASCR, .sbx)".as_ptr(),
    extensions: [c"sbx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAM_COUPE_COP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sam_coupe_cop".as_ptr(),
    mime_type: c"audio/x-cop".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"SAM Coupé E-Tracker Compiled SAA1099 Module (.cop / .etc)".as_ptr(),
    extensions: [c"cop".as_ptr(), c"etc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAMSUNG_PIT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"samsung_pit".as_ptr(),
    mime_type: c"application/x-samsung-pit".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Samsung Odin Partition Information Table (.pit)".as_ptr(),
    extensions: [c"pit".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SANKOM_OZD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sankom_ozd".as_ptr(),
    mime_type: c"application/x-sankom-ozd".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Sankom Audytor OZC/CO Heating & HVAC Engineering Project (.ozd) Sankom".as_ptr(),
    extensions: [c"ozd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAR_CAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sar_car".as_ptr(),
    mime_type: c"application/x-sar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SAR / CAR v2.0x Compressed Archive (.sar)".as_ptr(),
    extensions: [c"sar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sarc".as_ptr(),
    mime_type: c"application/x-nintendo-sarc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Wii U / 3DS / Switch SARC Resource Archive (.sarc)".as_ptr(),
    extensions: [c"sarc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sas".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"SAS dataset / transport".as_ptr(),
    extensions: [c"sas7bdat".as_ptr(), c"xpt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SAWGRASS_DM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sawgrass_dm".as_ptr(),
    mime_type: c"application/x-sawgrass-dm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sawgrass SG400/SG800 RPCS-R Printer Profile (.dm, Sawgrass Technologies)".as_ptr(),
    extensions: [c"dm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SBLIGHT_BOX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sblight_box".as_ptr(),
    mime_type: c"application/x-sbox-file".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Stormshield / Arkoon Security BOX Encrypted Container (.box)".as_ptr(),
    extensions: [c"box".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SBSTUDIO_PAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sbstudio_pac".as_ptr(),
    mime_type: c"audio/x-pac".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Henning Sauer SBStudio PAC Module (.pac)".as_ptr(),
    extensions: [c"pac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SC68_MUSIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sc68_music".as_ptr(),
    mime_type: c"audio/x-sc68".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Benjamin Gerard sc68 Atari ST / Amiga YM2149 & Paula Music (.sc68)".as_ptr(),
    extensions: [c"sc68".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SC_REP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sc_rep".as_ptr(),
    mime_type: c"application/x-sc-rep".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Blizzard StarCraft / StarCraft: Remastered Replay (.rep, 1998) Blizzard Entertainment".as_ptr(),
    extensions: [c"rep".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCALA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scala".as_ptr(),
    mime_type: c"application/x-scala".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Scala source".as_ptr(),
    extensions: [c"scala".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SCALEFORM_GFX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scaleform_gfx".as_ptr(),
    mime_type: c"application/x-scaleform-gfx".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Autodesk Scaleform GFx Movie (.gfx)".as_ptr(),
    extensions: [c"gfx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCC_MUSIXX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scc_musixx".as_ptr(),
    mime_type: c"audio/x-sng".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MSX Konami SCC-Musixx Song (.sng)".as_ptr(),
    extensions: [c"sng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCHINDLER_SZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"schindler_szip".as_ptr(),
    mime_type: c"application/x-szip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Michael Schindler SZIP Block-Sorted Stream (.sz)".as_ptr(),
    extensions: [c"sz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scp".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SuperCard Pro magnetic flux capture (.scp)".as_ptr(),
    extensions: [c"scp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scpt".as_ptr(),
    mime_type: c"application/x-applescript".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Apple Compiled AppleScript Bytecode (.scpt)".as_ptr(),
    extensions: [c"scpt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCRATCH_SB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scratch_sb".as_ptr(),
    mime_type: c"application/x-scratch-sb".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MIT Scratch 1.x Squeak Binary Project (.sb, 2007) MIT Media Lab".as_ptr(),
    extensions: [c"sb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCREENCAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"screencam".as_ptr(),
    mime_type: c"application/vnd.lotus-screencam".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Lotus ScreenCam Movie File (.scm, PRONOM fmt/1938)".as_ptr(),
    extensions: [c"scm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCRIPTWSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scriptwsf".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"scriptwsf".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCS_HASHFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scs_hashfs".as_ptr(),
    mime_type: c"application/x-scs-hashfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SCS Software HashFS Game Archive (.scs, 2012) SCS Software".as_ptr(),
    extensions: [c"scs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCS_PMD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scs_pmd".as_ptr(),
    mime_type: c"application/x-scs-pmd".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"SCS Software Prism3D 3D Model Descriptor (.pmd)".as_ptr(),
    extensions: [c"pmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCS_PMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scs_pmg".as_ptr(),
    mime_type: c"application/x-scs-pmg".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"SCS Software Prism3D Engine 3D Model Geometry (.pmg, 2012) SCS Software".as_ptr(),
    extensions: [c"pmg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCS_SII_ENC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scs_sii_enc".as_ptr(),
    mime_type: c"application/x-scs-sii-encrypted".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SCS Software Prism3D Encrypted SII Configuration / Save (.sii, 2012) SCS Software".as_ptr(),
    extensions: [c"sii".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCS_TOBJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scs_tobj".as_ptr(),
    mime_type: c"application/x-scs-tobj".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"SCS Software Prism3D Engine Texture Object Descriptor (.tobj, 2012) SCS Software".as_ptr(),
    extensions: [c"tobj".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scss".as_ptr(),
    mime_type: c"text/x-scss".as_ptr(),
    group: c"code".as_ptr(),
    description: c"SCSS source".as_ptr(),
    extensions: [c"scss".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SCUMM_AMIGA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scumm_amiga".as_ptr(),
    mime_type: c"audio/x-scumm-amiga".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Lucasfilm Games SCUMM Amiga Music Module (.scumm)".as_ptr(),
    extensions: [c"scumm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCUMM_CUTSCENE_CUT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scumm_cutscene_cut".as_ptr(),
    mime_type: c"application/x-scumm-cutscene-cut".as_ptr(),
    group: c"video".as_ptr(),
    description: c"LucasArts SCUMM Cutscene Resource (.cut)".as_ptr(),
    extensions: [c"cut".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCUMM_ROOM_LFL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scumm_room_lfl".as_ptr(),
    mime_type: c"application/x-scumm-room-lfl".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"LucasArts SCUMM v3 Room Resource File (.lfl, RO/HD/BX)".as_ptr(),
    extensions: [c"lfl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SCW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"scw".as_ptr(),
    mime_type: c"application/x-screenwriter".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Movie Magic Screenwriter Screenplay (.scw / .mms)".as_ptr(),
    extensions: [c"scw".as_ptr(), c"mms".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SDC_GDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sdc_gdb".as_ptr(),
    mime_type: c"application/x-sdc-gdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Esri Smart Data Compression (SDC FORMAT 1.00) Geospatial Database (.gdb, Esri)".as_ptr(),
    extensions: [c"gdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sds".as_ptr(),
    mime_type: c"application/x-lifetech-sds".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Applied Biosystems / Life Technologies SDS Real-Time PCR Data (.sds, PRONOM fmt/586)".as_ptr(),
    extensions: [c"sds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SDW2_SNP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sdw2_snp".as_ptr(),
    mime_type: c"application/x-sdw2-snp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SDW v2 System Snapshot (.snp)".as_ptr(),
    extensions: [c"snp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEAMA_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"seama_fw".as_ptr(),
    mime_type: c"application/x-seama-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Alpha Networks SEAMA Firmware Container (.seama)".as_ptr(),
    extensions: [c"seama".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEAN_CONRAN_SCR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sean_conran_scr".as_ptr(),
    mime_type: c"audio/x-sean-conran-scr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sean Conran Amiga Music Module (.scr)".as_ptr(),
    extensions: [c"scr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SECURITY_CFG_CCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"security_cfg_cch".as_ptr(),
    mime_type: c"application/x-security-cfg-cch".as_ptr(),
    group: c"application".as_ptr(),
    description: c".NET Code Access Security Config Cache (.config.cch)".as_ptr(),
    extensions: [c"config".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEGA_AFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sega_afs".as_ptr(),
    mime_type: c"application/x-sega-afs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sega Dreamcast / PS2 AFS File System Archive (.afs)".as_ptr(),
    extensions: [c"afs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEGA_FILM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sega_film".as_ptr(),
    mime_type: c"video/x-sega-film".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Sega Saturn FILM / Cinepak CPK Multimedia Container (.cpk / .cak)".as_ptr(),
    extensions: [c"cpk".as_ptr(), c"cak".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEGA_GSGM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sega_gsgm".as_ptr(),
    mime_type: c"model/x-sega-gsgm".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Sega Ryu Ga Gotoku Dragon Engine 3D Model (.gmd, 2016) Sega".as_ptr(),
    extensions: [c"gmd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEGY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"segy".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SEG-Y seismic data".as_ptr(),
    extensions: [c"segy".as_ptr(), c"sgy".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SELINUX_POLICY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"selinux_policy".as_ptr(),
    mime_type: c"application/x-selinux-policy".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NSA / Linux Kernel SELinux Binary Security Policy".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SELINUX_PP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"selinux_pp".as_ptr(),
    mime_type: c"application/x-selinux-policy-package".as_ptr(),
    group: c"application".as_ptr(),
    description: c"NSA SELinux Compiled Policy Module Package (.pp, 0x8FFF7CF9)".as_ptr(),
    extensions: [c"pp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEMI_OASIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"semi_oasis".as_ptr(),
    mime_type: c"application/x-semi-oasis".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"SEMI P39 OASIS Open Artwork System Interchange Standard IC Layout (.oas)".as_ptr(),
    extensions: [c"oas".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SENJUTSU_S7: MagikaTypeInfo = MagikaTypeInfo {
    label: c"senjutsu_s7".as_ptr(),
    mime_type: c"application/x-senjutsu-s7".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Senjutsu Tactical Campaign Map Data (SENJUTSUMAPDATA, .s7)".as_ptr(),
    extensions: [c"s7".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SERCOMM_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sercomm_fw".as_ptr(),
    mime_type: c"application/x-sercomm-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sercomm sErCoMm Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SERIF_PAGEPLUS_PUBLICATION_PPP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"serif_pageplus_publication_ppp".as_ptr(),
    mime_type: c"application/x-serif-pageplus-publication-ppp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Serif PagePlus Publication (.ppp, PRONOM fmt/1537)".as_ptr(),
    extensions: [c"ppp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SERPENT_STONE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"serpent_stone".as_ptr(),
    mime_type: c"application/x-stone-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Serpent OS / AerynOS Stone Binary Software Package (.stone, \\\\x00mos)".as_ptr(),
    extensions: [c"stone".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ses".as_ptr(),
    mime_type: c"application/x-cooledit-session".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Syntrillium Cool Edit / Adobe Audition Session (.ses)".as_ptr(),
    extensions: [c"ses".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SEVENZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sevenzip".as_ptr(),
    mime_type: c"application/x-7z-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"7-zip archive data".as_ptr(),
    extensions: [c"7z".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sff".as_ptr(),
    mime_type: c"application/x-sff".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Roche 454 Standard Flowgram Format (.sff, PRONOM fmt/504)".as_ptr(),
    extensions: [c"sff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SGC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sgc".as_ptr(),
    mime_type: c"audio/x-sgc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sega Master System / Game Gear / ColecoVision SGC Audio (.sgc)".as_ptr(),
    extensions: [c"sgc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SGI_MOVIE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sgi_movie".as_ptr(),
    mime_type: c"video/x-sgi-movie".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Silicon Graphics Movie Format (.mv/.movie, PRONOM fmt/1901)".as_ptr(),
    extensions: [c"mv".as_ptr(), c"movie".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SGI_RGB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sgi_rgb".as_ptr(),
    mime_type: c"image/x-sgi".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Silicon Graphics IRIS RGB Image (.sgi / .rgb / .bw)".as_ptr(),
    extensions: [c"sgi".as_ptr(), c"rgb".as_ptr(), c"bw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SGML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sgml".as_ptr(),
    mime_type: c"application/sgml".as_ptr(),
    group: c"text".as_ptr(),
    description: c"sgml".as_ptr(),
    extensions: [c"sgml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SH3_SMART: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sh3_smart".as_ptr(),
    mime_type: c"application/x-sh3-smart".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SH3 R02.04 Smart Configuration Profile (.smart)".as_ptr(),
    extensions: [c"smart".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SH3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sh3d".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"sh3d".as_ptr(),
    extensions: [c"sh3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHAPEFILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shapefile".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"gis".as_ptr(),
    description: c"ESRI Shapefile".as_ptr(),
    extensions: [c"dbf".as_ptr(), c"prj".as_ptr(), c"shp".as_ptr(), c"shx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHELL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shell".as_ptr(),
    mime_type: c"text/x-shellscript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Shell script".as_ptr(),
    extensions: [c"sh".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SHINMERA_SF3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shinmera_sf3".as_ptr(),
    mime_type: c"application/x-sf3".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Shinmera Simple File Format 3 Container (.sf3, \\\\x81SF3\\\\x00\\\\xE0\\\\xD0\\\\r\\\\n\\\\n)".as_ptr(),
    extensions: [c"sf3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shk".as_ptr(),
    mime_type: c"application/x-shk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"NuFX / ShrinkIt compressed archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shn".as_ptr(),
    mime_type: c"audio/x-shorten".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Shorten Lossless Audio (.shn)".as_ptr(),
    extensions: [c"shn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sho".as_ptr(),
    mime_type: c"audio/x-sho".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Shroom Player Module (.sho)".as_ptr(),
    extensions: [c"sho".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHOCKWAVE_W3D: MagikaTypeInfo = MagikaTypeInfo {
    label: c"shockwave_w3d".as_ptr(),
    mime_type: c"model/x-shockwave-w3d".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Intel / Macromedia Shockwave 3D Scene (IFX, .w3d, 2001) Macromedia / Adobe".as_ptr(),
    extensions: [c"w3d".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SHOWPARTNER_HLP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"showpartner_hlp".as_ptr(),
    mime_type: c"application/x-showpartner-hlp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Brightbill-Roberts Show Partner Animator Help File (.hlp)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SID: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sid".as_ptr(),
    mime_type: c"image/x-mrsid".as_ptr(),
    group: c"image".as_ptr(),
    description: c"LizardTech MrSID Multiresolution Seamless Image (.sid)".as_ptr(),
    extensions: [c"sid".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIDMON2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sidmon2".as_ptr(),
    mime_type: c"audio/x-sidmon2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Reinier van Vliet Amiga SidMon II Module (.sid2)".as_ptr(),
    extensions: [c"sid2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIERRA_AGI_VOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sierra_agi_vol".as_ptr(),
    mime_type: c"application/x-sierra-agi-vol".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sierra On-Line AGI Adventure Game Volume Archive (VOL.0, 1984) Sierra On-Line".as_ptr(),
    extensions: [c"vol".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIERRA_RBT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sierra_rbt".as_ptr(),
    mime_type: c"video/x-sierra-rbt".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Sierra On-Line SCI Robot Video Animation (.rbt)".as_ptr(),
    extensions: [c"rbt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIERRA_SCI32_PIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sierra_sci32_pic".as_ptr(),
    mime_type: c"image/x-sierra-sci32-pic".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sierra On-Line SCI1.1 / SCI32 Background Picture Resource (.p56, 1992) Sierra On-Line".as_ptr(),
    extensions: [c"p56".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIERRA_SCI32_VIEW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sierra_sci32_view".as_ptr(),
    mime_type: c"image/x-sierra-sci32-view".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sierra On-Line SCI32 Sprite View Resource (.v56, 1993) Sierra On-Line".as_ptr(),
    extensions: [c"v56".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIERRA_SCI_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sierra_sci_patch".as_ptr(),
    mime_type: c"audio/x-sierra-sci-patch".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sierra On-Line SCI0 AdLib Patch Bank (.003)".as_ptr(),
    extensions: [c"003".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIERRA_SOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sierra_sol".as_ptr(),
    mime_type: c"audio/x-sierra-sol".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sierra On-Line SOL Audio (.aud / .sfx)".as_ptr(),
    extensions: [c"aud".as_ptr(), c"sfx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIGMA_X3F: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sigma_x3f".as_ptr(),
    mime_type: c"image/x-sigma-x3f".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sigma Foveon X3 Camera RAW Image (.x3f, FOVb)".as_ptr(),
    extensions: [c"x3f".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIGNUM_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"signum_font".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Application Systems Heidelberg Signum! Atari ST Font (.e24 / .l30 / .p24 / .p9)".as_ptr(),
    extensions: [c"e24".as_ptr(), c"l30".as_ptr(), c"p24".as_ptr(), c"p9".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIGNUM_SDO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"signum_sdo".as_ptr(),
    mime_type: c"application/x-signum".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Application Systems Heidelberg Signum! 1/2 Atari ST Document (.sdo)".as_ptr(),
    extensions: [c"sdo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SILABS_GBL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"silabs_gbl".as_ptr(),
    mime_type: c"application/x-silabs-gbl".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Silicon Labs Gecko Bootloader Upgrade Image (.gbl) Silicon Labs".as_ptr(),
    extensions: [c"gbl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIMH_TAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"simh_tap".as_ptr(),
    mime_type: c"application/x-simh-tape".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SIMH / Ersatz-11 Magnetic Tape Image (.tap, 1993) Bob Supnik / John Wilson".as_ptr(),
    extensions: [c"tap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIMS1_IFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sims1_iff".as_ptr(),
    mime_type: c"application/x-sims1-iff".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Maxis The Sims 1 IFF 2.5 Resource Container (.iff / .spf / .flr / .wll, 2000) Maxis / Electronic Arts".as_ptr(),
    extensions: [c"iff".as_ptr(), c"spf".as_ptr(), c"flr".as_ptr(), c"wll".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SIMS4_THUMBNAIL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sims4_thumbnail".as_ptr(),
    mime_type: c"image/x-sims4-thumbnail".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Maxis / Electronic Arts The Sims 4 Tray Thumbnail (.hhi / .bpi / .sgi, 2014) Electronic Arts".as_ptr(),
    extensions: [c"hhi".as_ptr(), c"bpi".as_ptr(), c"sgi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SINCLAIR_QDOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sinclair_qdos".as_ptr(),
    mime_type: c"application/x-sinclair-qdos".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sinclair QL QDOS Filesystem Disk Image (QL5A / QL5B)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SINCLAIR_QL_F3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sinclair_ql_f3".as_ptr(),
    mime_type: c"application/x-sinclair-ql-f3".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Sinclair QL QLBAS Bitmap Font Resource (.f3)".as_ptr(),
    extensions: [c"f3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SINGBOX_SRS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"singbox_srs".as_ptr(),
    mime_type: c"application/x-singbox-srs".as_ptr(),
    group: c"application".as_ptr(),
    description: c"sing-box Binary Rule-Set Container (SRS v1–v5, .srs)".as_ptr(),
    extensions: [c"srs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SISX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sisx".as_ptr(),
    mime_type: c"application/vnd.symbian.install".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Symbian OS v9 SISX Installation Package (.sis / .sisx)".as_ptr(),
    extensions: [c"sis".as_ptr(), c"sisx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SKETCHUP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sketchup".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"SketchUp model".as_ptr(),
    extensions: [c"skp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SKOPS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"skops".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"skops model".as_ptr(),
    extensions: [c"skops".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SKS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sks".as_ptr(),
    mime_type: c"audio/x-sks".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Targhan / Arkos Amstrad CPC Starkos Song (.sks)".as_ptr(),
    extensions: [c"sks".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SLIM_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"slim_archive".as_ptr(),
    mime_type: c"application/x-slim-compressed".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"MS-DOS Dominic Herity SLIM Compressed File (.slm)".as_ptr(),
    extensions: [c"slm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SMACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"smacker".as_ptr(),
    mime_type: c"video/vnd.radgametools.smacker".as_ptr(),
    group: c"video".as_ptr(),
    description: c"RAD Game Tools Smacker Video Container (.smk)".as_ptr(),
    extensions: [c"smk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SMALI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"smali".as_ptr(),
    mime_type: c"application/x-smali".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Smali source".as_ptr(),
    extensions: [c"smali".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SMOOTHMOVE_PAN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"smoothmove_pan".as_ptr(),
    mime_type: c"image/x-smoothmove-pan".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Infinite Pictures SmoothMove Pan Viewer Panorama (.pan)".as_ptr(),
    extensions: [c"pan".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SMS_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sms_rom".as_ptr(),
    mime_type: c"application/x-sms-rom".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sega Master System / Game Gear Cartridge ROM (.sms / .gg)".as_ptr(),
    extensions: [c"sms".as_ptr(), c"gg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SN2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sn2".as_ptr(),
    mime_type: c"audio/x-sn2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Bluemoon Sound Club Module (.sn / .sn2)".as_ptr(),
    extensions: [c"sn".as_ptr(), c"sn2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SNAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"snap".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Snap archive".as_ptr(),
    extensions: [c"snap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SNEVENTRACKER_SNM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sneventracker_snm".as_ptr(),
    mime_type: c"audio/x-sneventracker".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"HertzDevil SnevenTracker Chiptune Module (.snm)".as_ptr(),
    extensions: [c"snm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SNGPLAY_OBSM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sngplay_obsm".as_ptr(),
    mime_type: c"audio/x-obsm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ObsM SNGPlay AdLib Chiptune (.sng)".as_ptr(),
    extensions: [c"sng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SNOOP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"snoop".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sun Microsystems Snoop packet capture (RFC 1761)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOF_FIRMWARE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sof_firmware".as_ptr(),
    mime_type: c"application/x-sof-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sound Open Firmware Audio DSP Image & Extended Manifest (.ri / .ldc)".as_ptr(),
    extensions: [c"ri".as_ptr(), c"ldc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOFTLIB_SLB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"softlib_slb".as_ptr(),
    mime_type: c"application/x-softlib-slb".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SoftLib Compressed Library Archive (.slb)".as_ptr(),
    extensions: [c"slb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOFTMAKER_DWR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"softmaker_dwr".as_ptr(),
    mime_type: c"application/x-softmaker-dwr".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SoftMaker Office UI & Dialog Resource Archive (.dwr)".as_ptr(),
    extensions: [c"dwr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOFTPLAN_SPV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"softplan_spv".as_ptr(),
    mime_type: c"application/x-softplan-spv".as_ptr(),
    group: c"document".as_ptr(),
    description: c"SoftPlan Architectural Design reView Project (.spv) SoftPlan Systems".as_ptr(),
    extensions: [c"spv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOLARIS_PKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"solaris_pkg".as_ptr(),
    mime_type: c"application/x-solaris-pkg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sun Solaris / SVR4 Package Datastream (.pkg)".as_ptr(),
    extensions: [c"pkg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOLIDITY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"solidity".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Solidity source".as_ptr(),
    extensions: [c"sol".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SONARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sonarc".as_ptr(),
    mime_type: c"audio/x-sonarc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sonarc Lossless Compressed Creative Voice Audio (.wv / .vc)".as_ptr(),
    extensions: [c"wv".as_ptr(), c"vc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONIX_INSTR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sonix_instr".as_ptr(),
    mime_type: c"audio/x-sonix-instr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Aegis Sonix Instrument Descriptor (.instr)".as_ptr(),
    extensions: [c"instr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONY_OMA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sony_oma".as_ptr(),
    mime_type: c"audio/x-sony-oma".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony OpenMG ATRAC3 Audio (.oma / .aa3, PRONOM fmt/2018)".as_ptr(),
    extensions: [c"oma".as_ptr(), c"aa3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONY_PICTUREGEAR_PTS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sony_picturegear_pts".as_ptr(),
    mime_type: c"application/x-sony-picturegear-pts".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sony PictureGear / VAIO Theme Package (PTM fnIA, .pts)".as_ptr(),
    extensions: [c"pts".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONY_PMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sony_pmp".as_ptr(),
    mime_type: c"image/x-sony-pmp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sony Cyber-shot DSC-F1 PMP Still Image (.pmp, 1996) Sony".as_ptr(),
    extensions: [c"pmp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONY_PSP_GIM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sony_psp_gim".as_ptr(),
    mime_type: c"image/x-psp-gim".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sony PlayStation Portable GIM Graphics Image (.gim, MIG.00.1PSP)".as_ptr(),
    extensions: [c"gim".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONY_PSP_GMO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sony_psp_gmo".as_ptr(),
    mime_type: c"model/x-sony-psp-gmo".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Sony PlayStation Portable GMO 3D Model (.gmo)".as_ptr(),
    extensions: [c"gmo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SONY_SGXD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sony_sgxd".as_ptr(),
    mime_type: c"application/x-sony-sgxd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony SCEI SGXD Sound Group Audio Container (.sgd / .sgb, 2006) Sony Computer Entertainment".as_ptr(),
    extensions: [c"sgd".as_ptr(), c"sgb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOPHOS_IDE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sophos_ide".as_ptr(),
    mime_type: c"application/x-sophos-ide".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Sophos Anti-Virus Virus Identity Definition (.ide, 1999) Sophos".as_ptr(),
    extensions: [c"ide".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOPROL_SPL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"soprol_spl".as_ptr(),
    mime_type: c"audio/x-soprol".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Ione Sound Programming Language Amiga Module (.spl)".as_ptr(),
    extensions: [c"spl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOUND_CONTROL_SC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sound_control_sc".as_ptr(),
    mime_type: c"audio/x-sound-control-sc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Holger Gehrmann Sound Control Amiga Module (.sc)".as_ptr(),
    extensions: [c"sc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOUND_IMAGES_SIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sound_images_sig".as_ptr(),
    mime_type: c"audio/x-sound-images-sig".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Tony Williams Sound Images Generation 2 AdLib Module (.sig)".as_ptr(),
    extensions: [c"sig".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOUNDFACTORY_PSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"soundfactory_psf".as_ptr(),
    mime_type: c"audio/x-soundfactory-psf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Advantec SoundFactory Amiga Synth Module (.psf)".as_ptr(),
    extensions: [c"psf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOUNDPLAYER_SJS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"soundplayer_sjs".as_ptr(),
    mime_type: c"audio/x-soundplayer-sjs".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Scott Johnston SoundPlayer Amiga Module (sjs.*)".as_ptr(),
    extensions: [c"sjs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOUNDSMITH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"soundsmith".as_ptr(),
    mime_type: c"audio/x-soundsmith".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Apple IIgs Huibert Aalbers SoundSmith Module (.sng)".as_ptr(),
    extensions: [c"sng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOUNDTRACKER_26: MagikaTypeInfo = MagikaTypeInfo {
    label: c"soundtracker_26".as_ptr(),
    mime_type: c"audio/x-soundtracker-26".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MnemoTroN Soundtracker 2.6 / Ice Tracker Amiga Module (.mod / .mtn)".as_ptr(),
    extensions: [c"mod".as_ptr(), c"mtn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SOX_AUDIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sox_audio".as_ptr(),
    mime_type: c"audio/x-sox".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sound eXchange (SoX) Native Audio Format (.sox)".as_ptr(),
    extensions: [c"sox".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPACE_RANGERS_GI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"space_rangers_gi".as_ptr(),
    mime_type: c"application/x-space-rangers-gi".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Elemental Games Space Rangers Graphics Image (.gi)".as_ptr(),
    extensions: [c"gi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPC700: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spc700".as_ptr(),
    mime_type: c"audio/x-spc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Super Nintendo SPC700 Sound Chip State (.spc)".as_ptr(),
    extensions: [c"spc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spd".as_ptr(),
    mime_type: c"application/x-font-speedo".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Bitstream Speedo Scalable Outline Font (.spd)".as_ptr(),
    extensions: [c"spd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPIRV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spirv".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"SPIR-V".as_ptr(),
    extensions: [c"spv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPIS_LH5: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spis_lh5".as_ptr(),
    mime_type: c"application/x-spis-lh5".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SPIS LH5-Compressed Container (Delphi TCompress, .lng)".as_ptr(),
    extensions: [c"lng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spm".as_ptr(),
    mime_type: c"audio/x-spm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Stone Amiga Stonetracker Module (.spm)".as_ptr(),
    extensions: [c"spm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPREADTRUM_DHTB_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spreadtrum_dhtb_img".as_ptr(),
    mime_type: c"application/x-spreadtrum-dhtb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Spreadtrum / Unisoc DHTB Signed Partition Image (DHTB)".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPREADTRUM_PAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spreadtrum_pac".as_ptr(),
    mime_type: c"application/x-spreadtrum-pac".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Spreadtrum / Unisoc PAC Firmware Package (.pac)".as_ptr(),
    extensions: [c"pac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPRITEPAD_SPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spritepad_spd".as_ptr(),
    mime_type: c"image/x-spritepad-spd".as_ptr(),
    group: c"document".as_ptr(),
    description: c"SpritePad C64 Sprite Image (.spd, PRONOM fmt/1561)".as_ptr(),
    extensions: [c"spd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spss".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"SPSS dataset".as_ptr(),
    extensions: [c"por".as_ptr(), c"sav".as_ptr(), c"zsav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SPU: MagikaTypeInfo = MagikaTypeInfo {
    label: c"spu".as_ptr(),
    mime_type: c"audio/x-spu".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony PlayStation SPU Register Dump (.spu)".as_ptr(),
    extensions: [c"spu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQEX_AKB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqex_akb".as_ptr(),
    mime_type: c"application/x-sqex-akb".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Square Enix AKB Audio Bank Container (.akb, 2010) Square Enix".as_ptr(),
    extensions: [c"akb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sql".as_ptr(),
    mime_type: c"application/x-sql".as_ptr(),
    group: c"code".as_ptr(),
    description: c"SQL source".as_ptr(),
    extensions: [c"sql".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SQLITE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqlite".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SQLITE database".as_ptr(),
    extensions: [c"sqlite".as_ptr(), c"sqlite3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQLITE2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqlite2".as_ptr(),
    mime_type: c"application/vnd.sqlite3".as_ptr(),
    group: c"database".as_ptr(),
    description: c"SQLite 2.1 database".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQLITE_JOURNAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqlite_journal".as_ptr(),
    mime_type: c"application/x-sqlite-journal".as_ptr(),
    group: c"database".as_ptr(),
    description: c"SQLite rollback journal".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQLITE_WAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqlite_wal".as_ptr(),
    mime_type: c"application/x-sqlite-journal".as_ptr(),
    group: c"database".as_ptr(),
    description: c"SQLite 3 Write-Ahead Log".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQUARE_ENIX_SCD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"square_enix_scd".as_ptr(),
    mime_type: c"audio/x-square-enix-scd".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Square Enix Sound Container Data (.scd)".as_ptr(),
    extensions: [c"scd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQUARE_ENIX_SPW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"square_enix_spw".as_ptr(),
    mime_type: c"audio/x-square-enix-spw".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Square Enix SeWave Audio Stream (.spw)".as_ptr(),
    extensions: [c"spw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQUASHFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"squashfs".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Squash filesystem".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQUIRREL_CNUT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"squirrel_cnut".as_ptr(),
    mime_type: c"application/x-squirrel-bytecode".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Squirrel VM Compiled Closure Bytecode (.cnut / .nut)".as_ptr(),
    extensions: [c"cnut".as_ptr(), c"nut".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SQZ_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sqz_archive".as_ptr(),
    mime_type: c"application/x-sqz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Jonas I. Hammarberg Squeeze It (SQZ) Archive (.sqz, 1990)".as_ptr(),
    extensions: [c"sqz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SRA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sra".as_ptr(),
    mime_type: c"application/x-ncbi-sra".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"NCBI Sequence Read Archive (.sra)".as_ptr(),
    extensions: [c"sra".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SRECORD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"srecord".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Motorola S-record firmware".as_ptr(),
    extensions: [c"s19".as_ptr(), c"srec".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SRT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"srt".as_ptr(),
    mime_type: c"text/srt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SubRip Text Format".as_ptr(),
    extensions: [c"srt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SSBU_PARACOBN_PRC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ssbu_paracobn_prc".as_ptr(),
    mime_type: c"application/x-ssbu-prc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Super Smash Bros. Ultimate Binary Parameter (paracobn, .prc)".as_ptr(),
    extensions: [c"prc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SSI_GFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ssi_gff".as_ptr(),
    mime_type: c"application/x-ssi-gff".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Strategic Simulations General File Format Archive (.gff)".as_ptr(),
    extensions: [c"gff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ST_DFUSE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"st_dfuse".as_ptr(),
    mime_type: c"application/x-st-dfuse".as_ptr(),
    group: c"application".as_ptr(),
    description: c"STMicroelectronics DfuSe USB Device Firmware Upgrade Image (.dfu)".as_ptr(),
    extensions: [c"dfu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STACKER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stacker".as_ptr(),
    mime_type: c"application/x-stacker".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Stac Electronics Stacker Compressed Disk Volume (.dsk)".as_ptr(),
    extensions: [c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STAD_PAC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stad_pac".as_ptr(),
    mime_type: c"image/x-stad-pac".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Atari ST STAD High-Resolution Compressed Graphic (.pac)".as_ptr(),
    extensions: [c"pac".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STALKER_XRAY_OGF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stalker_xray_ogf".as_ptr(),
    mime_type: c"application/x-stalker-xray-ogf".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"GSC Game World X-Ray Engine Compiled 3D Mesh (.ogf, v4)".as_ptr(),
    extensions: [c"ogf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STALKER_XRAY_THM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stalker_xray_thm".as_ptr(),
    mime_type: c"application/x-stalker-xray-thm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"GSC Game World X-Ray Engine Texture Descriptor (.thm, 0x0810)".as_ptr(),
    extensions: [c"thm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STARBOUND_PAK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"starbound_pak".as_ptr(),
    mime_type: c"application/x-starbound-pak".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Chucklefish Starbound SBAsset6 Game Archive (.pak, 2013) Chucklefish".as_ptr(),
    extensions: [c"pak".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STARTREKKER_NT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"startrekker_nt".as_ptr(),
    mime_type: c"audio/x-startrekker-nt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Fairlight Startrekker AM Synth Instrument Bank (.nt)".as_ptr(),
    extensions: [c"nt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STARWRITER_LEGACY_VOR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"starwriter_legacy_vor".as_ptr(),
    mime_type: c"application/x-starwriter-legacy-vor".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Star Division StarWriter DOS Legacy Document Template (.vor, 1991) Star Division".as_ptr(),
    extensions: [c"vor".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stata".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"scientific".as_ptr(),
    description: c"Stata dataset".as_ptr(),
    extensions: [c"dta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STATISTICA_SCR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"statistica_scr".as_ptr(),
    mime_type: c"application/x-statistica-scr".as_ptr(),
    group: c"database".as_ptr(),
    description: c"StatSoft Statistica Scrollsheet (.scr)".as_ptr(),
    extensions: [c"scr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STATISTICA_STA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"statistica_sta".as_ptr(),
    mime_type: c"application/x-statistica-sta".as_ptr(),
    group: c"database".as_ptr(),
    description: c"StatSoft Statistica Spreadsheet (.sta)".as_ptr(),
    extensions: [c"sta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STATISTICA_STG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"statistica_stg".as_ptr(),
    mime_type: c"application/x-statistica-stg".as_ptr(),
    group: c"image".as_ptr(),
    description: c"StatSoft Statistica Graph (.stg)".as_ptr(),
    extensions: [c"stg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stc".as_ptr(),
    mime_type: c"audio/x-stc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Sound Tracker Compiled Module (.stc)".as_ptr(),
    extensions: [c"stc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STEELMC_STLR_SRG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"steelmc_stlr_srg".as_ptr(),
    mime_type: c"application/x-steelmc-srg".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SteelMC Rust Minecraft Server Region Container (STLR, .srg)".as_ptr(),
    extensions: [c"srg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STEP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"step".as_ptr(),
    mime_type: c"model/step".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ISO 10303 STEP CAD model".as_ptr(),
    extensions: [c"step".as_ptr(), c"stp".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static STLBINARY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stlbinary".as_ptr(),
    mime_type: c"application/sla".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Stereolithography CAD (binary)".as_ptr(),
    extensions: [c"stl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STLTEXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stltext".as_ptr(),
    mime_type: c"application/sla".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Stereolithography CAD (text)".as_ptr(),
    extensions: [c"stl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static STOCKFISH_NNUE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stockfish_nnue".as_ptr(),
    mime_type: c"application/x-stockfish-nnue".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Stockfish Developers NNUE Chess Neural Network (.nnue)".as_ptr(),
    extensions: [c"nnue".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STONETRACKER_SPS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stonetracker_sps".as_ptr(),
    mime_type: c"audio/x-stonetracker-sps".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Amiga Stonetracker Sample Bank (.sps)".as_ptr(),
    extensions: [c"sps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STORM_ENGINE_TX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"storm_engine_tx".as_ptr(),
    mime_type: c"image/x-storm-engine-tx".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Akella Storm Engine Texture (.tx)".as_ptr(),
    extensions: [c"tx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STORM_GM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"storm_gm".as_ptr(),
    mime_type: c"application/x-storm-gm".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Akella Storm Engine 2.x 3D Geometry Model (.gm, 2003) Akella".as_ptr(),
    extensions: [c"gm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STOS_AMOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"stos_amos".as_ptr(),
    mime_type: c"application/x-amos-bank".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST STOS / Amiga AMOS BASIC Memory Bank (.mbk / .abk)".as_ptr(),
    extensions: [c"mbk".as_ptr(), c"abk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static STUDENT_WRITING_CENTER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"student_writing_center".as_ptr(),
    mime_type: c"application/x-student-writing-center".as_ptr(),
    group: c"document".as_ptr(),
    description: c"The Learning Company Student Writing Center Document (\\\\x1ATLC\\\\x01FF\\\\x00)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SUM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sum".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Checksum file".as_ptr(),
    extensions: [c"sum".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SUN_XRS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sun_xrs".as_ptr(),
    mime_type: c"application/x-sun-xrs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sun Microsystems XRS Resource Archive (.xrs) Sun Microsystems".as_ptr(),
    extensions: [c"xrs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SUNOS_TOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sunos_toc".as_ptr(),
    mime_type: c"application/x-sunos-toc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"SunOS 4.x XDR Distribution Tape Table of Contents (.toc, 1988) Sun Microsystems".as_ptr(),
    extensions: [c"toc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SUNRAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sunras".as_ptr(),
    mime_type: c"image/x-sun-raster".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Sun Raster image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SUNVOX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sunvox".as_ptr(),
    mime_type: c"audio/x-sunvox".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"WarmPlace SunVox Modular Synthesizer Project (.sunvox)".as_ptr(),
    extensions: [c"sunvox".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SUNXI_PACKAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"sunxi_package".as_ptr(),
    mime_type: c"application/x-sunxi-package".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Allwinner sunxi-package Boot Firmware Container (.fex, 2012) Allwinner Technology".as_ptr(),
    extensions: [c"fex".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SUPERSCAPE_VIRTUAL_REALITY_FORMAT_SVR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"superscape_virtual_reality_format_svr".as_ptr(),
    mime_type: c"application/x-superscape-virtual-reality-format-svr".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Superscape Virtual Reality Format (.svr, PRONOM fmt/734)".as_ptr(),
    extensions: [c"svr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SVD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"svd".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"svd".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SVG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"svg".as_ptr(),
    mime_type: c"image/svg+xml".as_ptr(),
    group: c"image".as_ptr(),
    description: c"SVG Scalable Vector Graphics image data".as_ptr(),
    extensions: [c"svg".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SVM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"svm".as_ptr(),
    mime_type: c"image/x-svm".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Borland VCL Metafile / StarView SVM (.svm / .vcl)".as_ptr(),
    extensions: [c"svm".as_ptr(), c"vcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SWF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"swf".as_ptr(),
    mime_type: c"application/x-shockwave-flash".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Small Web Format".as_ptr(),
    extensions: [c"swf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SWIFT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"swift".as_ptr(),
    mime_type: c"text/x-swift".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Swift".as_ptr(),
    extensions: [c"swift".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static SWITCH_PFS0: MagikaTypeInfo = MagikaTypeInfo {
    label: c"switch_pfs0".as_ptr(),
    mime_type: c"application/x-nintendo-pfs0".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Switch Partition FS0 / NSP Package (PFS0)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYMBOLICS_CARRY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"symbolics_carry".as_ptr(),
    mime_type: c"application/x-symbolics-carry".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Symbolics Lisp Machine Carry / Distribution Tape (.carry, 1982) Symbolics".as_ptr(),
    extensions: [c"carry".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYMLINK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"symlink".as_ptr(),
    mime_type: c"inode/symlink".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"Symbolic link".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYMMOD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"symmod".as_ptr(),
    mime_type: c"audio/x-symmod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Patrick Meng Amiga Symphonie Pro Module (.symmod)".as_ptr(),
    extensions: [c"symmod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYNDER_SNG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"synder_sng".as_ptr(),
    mime_type: c"audio/x-sng".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Synder SNG-Player Chiptune (.sng)".as_ptr(),
    extensions: [c"sng".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYNOPSE_SYNLZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"synopse_synlz".as_ptr(),
    mime_type: c"application/x-synopse-synlz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Synopse mORMot SynLZ Compressed Stream (.synlz, Arnaud Bouchez / Synopse)".as_ptr(),
    extensions: [c"synlz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYNTH_DREAM_SDR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"synth_dream_sdr".as_ptr(),
    mime_type: c"audio/x-synth-dream-sdr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Synth Dream Amiga Music Module (sdr.*)".as_ptr(),
    extensions: [c"sdr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYSTEMD_CATALOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"systemd_catalog".as_ptr(),
    mime_type: c"application/x-systemd-catalog".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Linux systemd Binary Journal Message Catalog (.catalog, 2012) Lennart Poettering".as_ptr(),
    extensions: [c"catalog".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static SYSTEMD_JOURNAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"systemd_journal".as_ptr(),
    mime_type: c"application/x-systemd-journal".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Linux systemd Structured Binary Journal Log (.journal)".as_ptr(),
    extensions: [c"journal".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static T98NEXT_NFD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"t98next_nfd".as_ptr(),
    mime_type: c"application/x-pc98-nfd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"NEC PC-9801 T98-Next Floppy Disk Image (.nfd, T98FDDIMAGE.R0/R1)".as_ptr(),
    extensions: [c"nfd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TADS3_IMAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tads3_image".as_ptr(),
    mime_type: c"application/x-t3vm-image".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"TADS 3 T3 VM Image (.t3)".as_ptr(),
    extensions: [c"t3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TAG_ADVENTURE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tag_adventure".as_ptr(),
    mime_type: c"application/x-tag-adventure".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"T.A.G. Text-Adventure-Generator Compiled Game (.tag)".as_ptr(),
    extensions: [c"tag".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TALLY_TCP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tally_tcp".as_ptr(),
    mime_type: c"application/x-tally-tcp".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Tally.ERP 9 Compiled Tally Definition Language (.tcp)".as_ptr(),
    extensions: [c"tcp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TANGOPRO_LIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tangopro_lib".as_ptr(),
    mime_type: c"application/x-tangopro-lib".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"ACCEL EDA / TangoPRO Component Library V2.0 (.lib)".as_ptr(),
    extensions: [c"lib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tar".as_ptr(),
    mime_type: c"application/x-tar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"POSIX tar archive".as_ptr(),
    extensions: [c"tar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TAXPREP_TFD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"taxprep_tfd".as_ptr(),
    mime_type: c"application/x-taxprep-tfd".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Wolters Kluwer Taxprep / Canadian CRA Tax Form Definition (.tfd)".as_ptr(),
    extensions: [c"tfd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TB_TBC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tb_tbc".as_ptr(),
    mime_type: c"application/x-turbo-basic-tbc".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland Turbo Basic Chain Module (.tbc)".as_ptr(),
    extensions: [c"tbc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TB_TBH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tb_tbh".as_ptr(),
    mime_type: c"application/x-turbo-basic-tbh".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Borland Turbo Basic Help File (.tbh)".as_ptr(),
    extensions: [c"tbh".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TCL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tcl".as_ptr(),
    mime_type: c"application/x-tcl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Tcl".as_ptr(),
    extensions: [c"tcl".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TD0: MagikaTypeInfo = MagikaTypeInfo {
    label: c"td0".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Sydex TeleDisk floppy image (.td0)".as_ptr(),
    extensions: [c"td0".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TDBD_DATABASE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tdbd_database".as_ptr(),
    mime_type: c"application/x-tdbd-database".as_ptr(),
    group: c"database".as_ptr(),
    description: c"TDBD Binary Database (.tdbd)".as_ptr(),
    extensions: [c"tdbd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tds".as_ptr(),
    mime_type: c"application/x-debug-symbols".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland Turbo Debugger Symbols (.tds)".as_ptr(),
    extensions: [c"tds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TEK_WFM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tek_wfm".as_ptr(),
    mime_type: c"application/x-tektronix-wfm".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Tektronix Oscilloscope Reference Waveform (.wfm, :WFM#001/002/003)".as_ptr(),
    extensions: [c"wfm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TELEGRAM_TDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"telegram_tdf".as_ptr(),
    mime_type: c"application/x-telegram-tdf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Telegram Desktop Encrypted Data File (TDF$)".as_ptr(),
    extensions: [c"tdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TELIX_SALT_SLC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"telix_salt_slc".as_ptr(),
    mime_type: c"application/x-telix-salt-slc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Telix SALT Compiled Script (.slc, 1988, Exis / deltaComm)".as_ptr(),
    extensions: [c"slc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TENCENT_TAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tencent_tap".as_ptr(),
    mime_type: c"image/x-tencent-tap".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Tencent TAP Animated Image Container (.tap)".as_ptr(),
    extensions: [c"tap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TENSORFLOWJS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tensorflowjs".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TensorFlow.js model".as_ptr(),
    extensions: [c"json".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TERRARIA_RELOGIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"terraria_relogic".as_ptr(),
    mime_type: c"application/x-terraria-relogic".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Re-Logic Terraria Binary World / Player / Map File (.wld / .plr / .map, 2011) Re-Logic".as_ptr(),
    extensions: [c"wld".as_ptr(), c"plr".as_ptr(), c"map".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TERRARIA_TMOD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"terraria_tmod".as_ptr(),
    mime_type: c"application/x-terraria-tmod".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"tModLoader Terraria Compiled Mod Archive (.tmod, 2015) tModLoader".as_ptr(),
    extensions: [c"tmod".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TEXTPROTO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"textproto".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Text protocol buffer".as_ptr(),
    extensions: [c"textproto".as_ptr(), c"textpb".as_ptr(), c"pbtxt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TFE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tfe".as_ptr(),
    mime_type: c"audio/x-tfe".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Shiru TFM Music Maker YM2203/YM2612 Module (.tfe)".as_ptr(),
    extensions: [c"tfe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TFLITE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tflite".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TensorFlow Lite / LiteRT".as_ptr(),
    extensions: [c"tflite".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TFM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tfm".as_ptr(),
    mime_type: c"application/x-tex-tfm".as_ptr(),
    group: c"font".as_ptr(),
    description: c"TeX Font Metrics (.tfm)".as_ptr(),
    extensions: [c"tfm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TFMX_MOD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tfmx_mod".as_ptr(),
    mime_type: c"audio/x-tfmx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Chris Huelsbeck TFMX (The Final Musicsystem eXtended) Amiga Module (.mdat)".as_ptr(),
    extensions: [c"mdat".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TFRECORD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tfrecord".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TensorFlow TFRecord".as_ptr(),
    extensions: [c"tfrecord".as_ptr(), c"tfrecords".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TGA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tga".as_ptr(),
    mime_type: c"image/x-tga".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Targa image data".as_ptr(),
    extensions: [c"tga".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TGCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tgcf".as_ptr(),
    mime_type: c"application/x-setup-specialist-tgcf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"TG-Soft Setup Specialist Multi-Part Archive Volume (.2 / .3, 1999) Bernd Steinhauser".as_ptr(),
    extensions: [c"2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static THEBAT_SRB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"thebat_srb".as_ptr(),
    mime_type: c"application/x-thebat-srb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Ritlabs The Bat! Sorting Office Filter Rules (ACCOUNT.SRB)".as_ptr(),
    extensions: [c"srb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static THEDRAW_TDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"thedraw_tdf".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"TheSoft TheDraw ANSI Block / Outline Font (.tdf)".as_ptr(),
    extensions: [c"tdf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static THILMERA7_ZPAC_T7P: MagikaTypeInfo = MagikaTypeInfo {
    label: c"thilmera7_zpac_t7p".as_ptr(),
    mime_type: c"application/x-thilmera7-zpac-t7p".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"thilmera7 System Monitor ZPAC Resource Package (.t7p)".as_ptr(),
    extensions: [c"t7p".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static THUMBSDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"thumbsdb".as_ptr(),
    mime_type: c"image/vnd.ms-thumb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows thumbnail cache".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_AIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_ais".as_ptr(),
    mime_type: c"application/x-ti-ais".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Texas Instruments Application Image Script Boot Image (.ais) Texas Instruments".as_ptr(),
    extensions: [c"ais".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_BTS_SCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_bts_script".as_ptr(),
    mime_type: c"application/x-ti-bts-script".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Texas Instruments WiLink Bluetooth Init Script (.bts, BTSB)".as_ptr(),
    extensions: [c"bts".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_MULTICORE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_multicore".as_ptr(),
    mime_type: c"application/x-ti-multicore".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Texas Instruments Multicore Meta Header Application Image (.appimage) Texas Instruments".as_ptr(),
    extensions: [c"appimage".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_OAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_oad".as_ptr(),
    mime_type: c"application/x-ti-oad".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Texas Instruments SimpleLink Over-the-Air Download Firmware Image (.bin) Texas Instruments".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_RPRC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_rprc".as_ptr(),
    mime_type: c"application/x-ti-rprc".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Texas Instruments Remote Processor Resource Core Firmware Image (.rprc) Texas Instruments".as_ptr(),
    extensions: [c"rprc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_SMARTAMP_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_smartamp_fw".as_ptr(),
    mime_type: c"application/x-ti-smartamp-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Texas Instruments TAS2781 / TAS2563 Smart Amplifier DSP Firmware (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TI_TPS_UPG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ti_tps_upg".as_ptr(),
    mime_type: c"application/x-ti-tps-upg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Dell / TI TPS65987 USB-C Monitor Firmware Upgrade (.upg, Texas Instruments)".as_ptr(),
    extensions: [c"upg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TIFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tiff".as_ptr(),
    mime_type: c"image/tiff".as_ptr(),
    group: c"image".as_ptr(),
    description: c"TIFF image data".as_ptr(),
    extensions: [c"tiff".as_ptr(), c"tif".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TIME_OF_WAR_D3E: MagikaTypeInfo = MagikaTypeInfo {
    label: c"time_of_war_d3e".as_ptr(),
    mime_type: c"application/x-time-of-war-d3e".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"The Time of War Engine Data File v4.00 (.d3e)".as_ptr(),
    extensions: [c"d3e".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TLS_SESSION_NET443: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tls_session_net443".as_ptr(),
    mime_type: c"application/x-tls-session-net443".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ASN.1 DER SSL/TLS Session Cache Entry (.net_443, OpenSSL SSL_SESSION)".as_ptr(),
    extensions: [c"net_443".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TMDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tmdx".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"tmdx".as_ptr(),
    extensions: [c"tmdx".as_ptr(), c"tmvx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TNEF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tnef".as_ptr(),
    mime_type: c"application/vnd.ms-tnef".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Transport Neutral Encapsulation Format (winmail.dat / .tnef)".as_ptr(),
    extensions: [c"tnef".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"toml".as_ptr(),
    mime_type: c"application/toml".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Tom's obvious, minimal language".as_ptr(),
    extensions: [c"toml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TOMOFON_TOMODEL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tomofon_tomodel".as_ptr(),
    mime_type: c"audio/x-tomofon-tomodel".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Klevgränd Tomofon Audio Model (.tomodel)".as_ptr(),
    extensions: [c"tomodel".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOMOFON_TOPATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tomofon_topatch".as_ptr(),
    mime_type: c"audio/x-tomofon-topatch".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Klevgränd Tomofon Audio Patch (velKqdiP, .topatch)".as_ptr(),
    extensions: [c"topatch".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOMY_TRACKER_SG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tomy_tracker_sg".as_ptr(),
    mime_type: c"audio/x-tomy-tracker-sg".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Tomy Tracker Amiga Music Module (.sg)".as_ptr(),
    extensions: [c"sg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOPS10_BACKUP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tops10_backup".as_ptr(),
    mime_type: c"application/x-tops10-backup".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DEC TOPS-10 BACKUP / Interchange Tape Stream (.bck, 1975) Digital Equipment".as_ptr(),
    extensions: [c"bck".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOPS20_DUMPER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tops20_dumper".as_ptr(),
    mime_type: c"application/x-tops20-dumper".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DEC TOPS-20 DUMPER 36-Bit Tape Saveset (.dumper / .tap, 1976) Digital Equipment".as_ptr(),
    extensions: [c"dumper".as_ptr(), c"tap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOPSOLID_RCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"topsolid_rcf".as_ptr(),
    mime_type: c"application/x-topsolid-rcf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Missler TopSolid MFC CArchive Resource Configuration (.rcf)".as_ptr(),
    extensions: [c"rcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOPSPEED_TPS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"topspeed_tps".as_ptr(),
    mime_type: c"application/x-topspeed-tps".as_ptr(),
    group: c"database".as_ptr(),
    description: c"SoftVelocity Clarion TopSpeed Database (.tps)".as_ptr(),
    extensions: [c"tps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TORCHSCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"torchscript".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TorchScript archive".as_ptr(),
    extensions: [c"pt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TORQUESCRIPT_DSO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"torquescript_dso".as_ptr(),
    mime_type: c"application/x-torquescript-dso".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"GarageGames Torque Game Engine Compiled TorqueScript (.dso, 2001) GarageGames".as_ptr(),
    extensions: [c"dso".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TORRENT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"torrent".as_ptr(),
    mime_type: c"application/x-bittorrent".as_ptr(),
    group: c"application".as_ptr(),
    description: c"BitTorrent file".as_ptr(),
    extensions: [c"torrent".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOTAL_WAR_PFH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"total_war_pfh".as_ptr(),
    mime_type: c"application/x-totalwar-pack".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Creative Assembly Total War PackFile Archive (.pack, PFH0..PFH5)".as_ptr(),
    extensions: [c"pack".as_ptr(), c"pfh5".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TOYOTA_CUW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"toyota_cuw".as_ptr(),
    mime_type: c"application/x-toyota-cuw".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Toyota / Lexus Techstream Calibration Update Wizard Package (.cuw) Toyota".as_ptr(),
    extensions: [c"cuw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TP_OVR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tp_ovr".as_ptr(),
    mime_type: c"application/x-turbo-pascal-ovr".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Borland Turbo Pascal Overlay File (.ovr)".as_ptr(),
    extensions: [c"ovr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TPLINK_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tplink_fw".as_ptr(),
    mime_type: c"application/x-tplink-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TP-LINK Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TPLINK_MINIFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tplink_minifs".as_ptr(),
    mime_type: c"application/x-tplink-minifs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"TP-LINK MiniFS Embedded Read-Only Filesystem (.minifs)".as_ptr(),
    extensions: [c"minifs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRACKERPACKER_MEXX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"trackerpacker_mexx".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Crazy Crack TrackerPacker Compressed Amiga Module (MEXX_TP / CPLX_TP3)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRACKMANIA_GBX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"trackmania_gbx".as_ptr(),
    mime_type: c"application/x-nadeo-gbx".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nadeo GameBox Binary Asset & TrackMania Map (.gbx, 2003) Nadeo / Ubisoft".as_ptr(),
    extensions: [c"gbx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRAINZ_CDP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"trainz_cdp".as_ptr(),
    mime_type: c"application/x-trainz-cdp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Auran / N3V Games Trainz Content Dispatcher Pack (.cdp, 2001) N3V Games".as_ptr(),
    extensions: [c"cdp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRAINZ_IM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"trainz_im".as_ptr(),
    mime_type: c"model/x-trainz-im".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Auran Trainz Jet Indexed Mesh (.im)".as_ptr(),
    extensions: [c"im".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRESORIT_PROFILE_BACKUP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tresorit_profile_backup".as_ptr(),
    mime_type: c"application/x-tresorit-profile-backup".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Tresorit Encrypted Storage Profile Backup (.backup)".as_ptr(),
    extensions: [c"backup".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TROC_FS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"troc_fs".as_ptr(),
    mime_type: c"application/x-troc-fs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Toshiba TROC Read-Only Embedded Filesystem (.troc)".as_ptr(),
    extensions: [c"troc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRPH_XPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"trph_xpd".as_ptr(),
    mime_type: c"application/x-trph-xpd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TRPH Mobile Printer Configuration (.xpd)".as_ptr(),
    extensions: [c"xpd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TRUE_COLOUR_PICTURE_SPOOKY_TRP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"true_colour_picture_spooky_trp".as_ptr(),
    mime_type: c"application/x-true-colour-picture-spooky-trp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Atari Falcon Spooky Sprites True Colour Picture (.trp, PRONOM fmt/1607)".as_ptr(),
    extensions: [c"trp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TSOUNDSYSTEM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tsoundsystem".as_ptr(),
    mime_type: c"audio/x-tsoundsystem".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"T'SoundSystem Tracker Module (.tsd)".as_ptr(),
    extensions: [c"tsd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TSV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tsv".as_ptr(),
    mime_type: c"text/tab-separated-values".as_ptr(),
    group: c"code".as_ptr(),
    description: c"TSV document".as_ptr(),
    extensions: [c"tsv".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TTC_XCFG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ttc_xcfg".as_ptr(),
    mime_type: c"application/x-ttc-xcfg".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TCW40 Controller Configuration (.xcfg)".as_ptr(),
    extensions: [c"xcfg".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ttf".as_ptr(),
    mime_type: c"font/sfnt".as_ptr(),
    group: c"font".as_ptr(),
    description: c"TrueType Font data".as_ptr(),
    extensions: [c"ttf".as_ptr(), c"ttc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TUNDRA_TND: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tundra_tnd".as_ptr(),
    mime_type: c"application/x-tundra-tnd".as_ptr(),
    group: c"document".as_ptr(),
    description: c"TundraDraw ANSI/ASCII Text Art (.tnd, PRONOM fmt/1603)".as_ptr(),
    extensions: [c"tnd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TUNEFISH2_TF2M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tunefish2_tf2m".as_ptr(),
    mime_type: c"audio/x-tunefish2".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Brain Control Tunefish 2 Synth Module (.tf2m)".as_ptr(),
    extensions: [c"tf2m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TUNEUP_TUN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tuneup_tun".as_ptr(),
    mime_type: c"application/x-tuneup-tun".as_ptr(),
    group: c"application".as_ptr(),
    description: c"TuneUp Utilities Installation/Undo Log (.tun, 2004) TuneUp Software".as_ptr(),
    extensions: [c"tun".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TURBOPOWER_POPHELP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"turbopower_pophelp".as_ptr(),
    mime_type: c"application/x-turbopower-pophelp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"TurboPower PopHelp / Object Professional Help Database (.hlp, 1990)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TURBOPOWER_TPHELP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"turbopower_tphelp".as_ptr(),
    mime_type: c"application/x-turbopower-tphelp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"TurboPower Turbo Professional Reference Manual / TPHelp Database (.hlp, 1989)".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TURBOPOWER_TPHELP2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"turbopower_tphelp2".as_ptr(),
    mime_type: c"application/x-turbopower-tph2".as_ptr(),
    group: c"document".as_ptr(),
    description: c"TurboPower Object Professional Compiled Help (.hlp, 1991) TurboPower Software".as_ptr(),
    extensions: [c"hlp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TURVISION_RWP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"turvision_rwp".as_ptr(),
    mime_type: c"application/x-tvrw-resource".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Turbo Vision Resource Workshop Project (.rwp, 1993) BitSoft / Borland".as_ptr(),
    extensions: [c"rwp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TWFB_TSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"twfb_tsf".as_ptr(),
    mime_type: c"application/x-ultimate-ride-tsf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Disney / Walt Disney Imagineering Ultimate Ride Text Strings (.tsf)".as_ptr(),
    extensions: [c"tsf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TWIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"twig".as_ptr(),
    mime_type: c"text/x-twig".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Twig template".as_ptr(),
    extensions: [c"twig".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TWIN_TRACK_DMO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"twin_track_dmo".as_ptr(),
    mime_type: c"audio/x-twin-track-dmo".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"TwinTeam Twin TrackPlayer Compressed OPL2 Module (.dmo)".as_ptr(),
    extensions: [c"dmo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TWINVQ_VQF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"twinvq_vqf".as_ptr(),
    mime_type: c"audio/x-twinvq".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"NTT Human Interface Labs TwinVQ Transform-Domain Audio (.vqf)".as_ptr(),
    extensions: [c"vqf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TXF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"txf".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"PLIB / Mark Kilgard OpenGL Textured Font (.txf)".as_ptr(),
    extensions: [c"txf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TXT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"txt".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Generic text document".as_ptr(),
    extensions: [c"txt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TYPESCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"typescript".as_ptr(),
    mime_type: c"application/typescript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"TypeScript source".as_ptr(),
    extensions: [c"ts".as_ptr(), c"mts".as_ptr(), c"cts".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static TZIF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tzif".as_ptr(),
    mime_type: c"application/tzif".as_ptr(),
    group: c"application".as_ptr(),
    description: c"IANA Time Zone Information Binary Format (TZif)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static TZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"tzip".as_ptr(),
    mime_type: c"application/x-tzip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"TZIP Compressed Stream (.tz)".as_ptr(),
    extensions: [c"tz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UA_RESUME1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ua_resume1".as_ptr(),
    mime_type: c"application/x-ua-resume1".as_ptr(),
    group: c"application".as_ptr(),
    description: c"UA Developments Patcher Download Resume State (UA-RESUME-1, .resume)".as_ptr(),
    extensions: [c"resume".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBF_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubf_font".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Symantec Time Line Universal BitMap Font (UBF92A .fnt)".as_ptr(),
    extensions: [c"fnt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBICOM_FW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubicom_fw".as_ptr(),
    mime_type: c"application/x-ubicom-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Ubicom IP3K / IP5K / IP7K Network Processor Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBIFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubifs".as_ptr(),
    mime_type: c"application/x-ubifs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Linux Unsorted Block Image / UBIFS Flash Filesystem (.ubi / .ubifs)".as_ptr(),
    extensions: [c"ubi".as_ptr(), c"ubifs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBIQUITI_UBNT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubiquiti_ubnt".as_ptr(),
    mime_type: c"application/x-ubiquiti-firmware".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Ubiquiti airOS / EdgeOS Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBISOFT_ANVIL_ACV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubisoft_anvil_acv".as_ptr(),
    mime_type: c"application/x-ubisoft-anvil-acv".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ubisoft Anvil Engine Save Game Container (.acv)".as_ptr(),
    extensions: [c"acv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBISOFT_JADE_BF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubisoft_jade_bf".as_ptr(),
    mime_type: c"application/x-ubisoft-jade-bf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Ubisoft Jade Engine Bigfile Archive (.bf, 2003) Ubisoft Montpellier".as_ptr(),
    extensions: [c"bf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBISOFT_XBT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ubisoft_xbt".as_ptr(),
    mime_type: c"application/x-ubisoft-xbt".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Ubisoft Dunia Engine XBT Texture (.xbt, 2008) Ubisoft Montreal".as_ptr(),
    extensions: [c"xbt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UBOOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uboot".as_ptr(),
    mime_type: c"application/x-uboot-image".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Das U-Boot legacy uImage firmware".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UCLINUX_BFLT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uclinux_bflt".as_ptr(),
    mime_type: c"application/x-bflt".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"uClinux Binary Flat Format Executable (bFLT)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UCSD_CODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ucsd_code".as_ptr(),
    mime_type: c"application/x-ucsd-code".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"UCSD Pascal p-Machine Compiled Codefile (.code, 1977) UC San Diego / SofTech Microsystems".as_ptr(),
    extensions: [c"code".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UCSD_VOL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ucsd_vol".as_ptr(),
    mime_type: c"application/x-ucsd-volume".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"UCSD Pascal p-System Disk Volume (.vol / .dsk, 1977) UC San Diego / SofTech Microsystems".as_ptr(),
    extensions: [c"vol".as_ptr(), c"dsk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UDF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"udf".as_ptr(),
    mime_type: c"application/x-udf-image".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Universal Disc Format".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UEFI_CAPSULE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uefi_capsule".as_ptr(),
    mime_type: c"application/x-uefi-capsule".as_ptr(),
    group: c"application".as_ptr(),
    description: c"UEFI Update Capsule / AMI Aptio Signed Firmware Capsule (.cap / .bin) UEFI Forum".as_ptr(),
    extensions: [c"cap".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UEFI_FV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uefi_fv".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"UEFI firmware volume".as_ptr(),
    extensions: [c"fd".as_ptr(), c"fv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UEFI_TE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uefi_te".as_ptr(),
    mime_type: c"application/x-uefi-te".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"UEFI Terse Executable (TE) firmware image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UF2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uf2".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"UF2 firmware".as_ptr(),
    extensions: [c"uf2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UHARC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uharc".as_ptr(),
    mime_type: c"application/x-uharc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"UHARC high-compression archive (.uha)".as_ptr(),
    extensions: [c"uha".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UJAM_BEATMAKER_BLOB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ujam_beatmaker_blob".as_ptr(),
    mime_type: c"audio/x-ujam-beatmaker-blob".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"UJAM Beatmaker Instrument Sample Bank (.blob)".as_ptr(),
    extensions: [c"blob".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ULT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ult".as_ptr(),
    mime_type: c"audio/x-ult".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"MAS UltraTracker Module (.ult)".as_ptr(),
    extensions: [c"ult".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ULTIMA6_M: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ultima6_m".as_ptr(),
    mime_type: c"audio/x-ultima6-m".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Origin Systems Ultima VI AdLib Music (.m)".as_ptr(),
    extensions: [c"m".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNDEFINED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"undefined".as_ptr(),
    mime_type: c"application/undefined".as_ptr(),
    group: c"undefined".as_ptr(),
    description: c"Undefined".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNI_MIKMOD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uni_mikmod".as_ptr(),
    mime_type: c"audio/x-uni".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Jean-Paul Mikkers MikMod UniMod Module (.uni)".as_ptr(),
    extensions: [c"uni".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNIFLEX_IMG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"uniflex_img".as_ptr(),
    mime_type: c"application/x-uniflex-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Tektronix 4404 / TSC UniFLEX Disk Image (.img, 1984) Tektronix / Technical Systems Consultants".as_ptr(),
    extensions: [c"img".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNIQUE_DEV_UDS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unique_dev_uds".as_ptr(),
    mime_type: c"audio/x-unique-dev-uds".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Unique Development Amiga Music Module (.uds)".as_ptr(),
    extensions: [c"uds".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNITY_ADDRESSABLES_CATALOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unity_addressables_catalog".as_ptr(),
    mime_type: c"application/x-unity-addressables-catalog".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Unity Engine Addressables Binary Content Catalog (catalog.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNITY_ASSETS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unity_assets".as_ptr(),
    mime_type: c"application/x-unity-assets".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Unity Engine Serialized Asset File (.assets / globalgamemanagers / level0)".as_ptr(),
    extensions: [c"assets".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNITY_ENLIGHTEN_CAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unity_enlighten_caw".as_ptr(),
    mime_type: c"application/x-unity-enlighten-caw".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Unity Engine Enlighten Precomputed GI Cluster/Albedo Workspace (.caw / .sse)".as_ptr(),
    extensions: [c"caw".as_ptr(), c"sse".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNITY_ENLIGHTEN_VIS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unity_enlighten_vis".as_ptr(),
    mime_type: c"application/x-unity-enlighten-vis".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Unity Engine Enlighten Precomputed Scene Visibility Data (GEVS, .vis)".as_ptr(),
    extensions: [c"vis".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNITY_PREVIEW_INFO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unity_preview_info".as_ptr(),
    mime_type: c"application/x-unity-preview-info".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Unity Editor Library Asset Preview Descriptor (PreviewAssetData, .info)".as_ptr(),
    extensions: [c"info".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNITY_WEBDATA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unity_webdata".as_ptr(),
    mime_type: c"application/x-unity-webdata".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Unity Engine WebGL Data Package (.data / .unityweb, UnityWebData1.0 / TuanjieWebData1.0)".as_ptr(),
    extensions: [c"data".as_ptr(), c"unityweb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNIXCOMPRESS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unixcompress".as_ptr(),
    mime_type: c"application/x-compress".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"unixcompress".as_ptr(),
    extensions: [c"z".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNIXWARE_BFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unixware_bfs".as_ptr(),
    mime_type: c"application/x-unixware-bfs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"SCO UnixWare Boot File System Image (.bfs)".as_ptr(),
    extensions: [c"bfs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNKNOWN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unknown".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Unknown binary data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_GVAS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_gvas".as_ptr(),
    mime_type: c"application/x-unreal-gvas".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Epic Games Unreal Engine GVAS SaveGame (.sav)".as_ptr(),
    extensions: [c"sav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_LOCMETA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_locmeta".as_ptr(),
    mime_type: c"application/x-unreal-locmeta".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Epic Games Unreal Engine Localization Metadata (.locmeta)".as_ptr(),
    extensions: [c"locmeta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_LOCRES: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_locres".as_ptr(),
    mime_type: c"application/x-unreal-locres".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Epic Games Unreal Engine Localization Resource (.locres)".as_ptr(),
    extensions: [c"locres".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_MANIFEST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_manifest".as_ptr(),
    mime_type: c"application/x-unreal-manifest".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Epic Games Unreal Engine BuildPatch Binary Manifest".as_ptr(),
    extensions: [c"buildpatch".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_PKG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_pkg".as_ptr(),
    mime_type: c"application/x-unreal-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Epic Games Unreal Engine Package / Asset (.uasset / .umap / .upk / .u)".as_ptr(),
    extensions: [c"uasset".as_ptr(), c"umap".as_ptr(), c"upk".as_ptr(), c"u".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_PSO_CACHE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_pso_cache".as_ptr(),
    mime_type: c"application/x-unreal-pipelinecache".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Epic Games Unreal Engine Pipeline State Object Cache (.upipelinecache)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_USMAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_usmap".as_ptr(),
    mime_type: c"application/x-unreal-usmap".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Unreal Engine 4/5 Unversioned Property Schema Mapping (.usmap, 2020) Epic Games / CUE4Parse".as_ptr(),
    extensions: [c"usmap".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_UTOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_utoc".as_ptr(),
    mime_type: c"application/x-unreal-utoc".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Epic Games Unreal Engine 4.26 / 5 IoStore Table of Contents (.utoc)".as_ptr(),
    extensions: [c"utoc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_UTRACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_utrace".as_ptr(),
    mime_type: c"application/x-unreal-utrace".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Epic Games Unreal Insights Trace Capture (.utrace)".as_ptr(),
    extensions: [c"utrace".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNREAL_UZ2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unreal_uz2".as_ptr(),
    mime_type: c"application/x-unreal-uz2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Unreal Engine 2 Redirect Compressed Package (.uz2, 2002) Epic Games".as_ptr(),
    extensions: [c"uz2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UNSUPPORTED: MagikaTypeInfo = MagikaTypeInfo {
    label: c"unsupported".as_ptr(),
    mime_type: c"inode/x-unsupported".as_ptr(),
    group: c"inode".as_ptr(),
    description: c"Unsupported file type".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static USD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"usd".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Universal Scene Description".as_ptr(),
    extensions: [c"usd".as_ptr(), c"usda".as_ptr(), c"usdc".as_ptr(), c"usdz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static UTAU_FRQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"utau_frq".as_ptr(),
    mime_type: c"application/x-utau-frq".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"UTAU Singing Synthesizer Fundamental Frequency Table (.frq, 2008) Ameya/Ayame".as_ptr(),
    extensions: [c"frq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static V129_RDT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"v129_rdt".as_ptr(),
    mime_type: c"application/x-v129-rdt".as_ptr(),
    group: c"application".as_ptr(),
    description: c"DMR Radio Codeplug Data (V129 / V100, .rdt)".as_ptr(),
    extensions: [c"rdt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static V8_BYTECODE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"v8_bytecode".as_ptr(),
    mime_type: c"application/x-v8-bytecode".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Google V8 JavaScript Engine Cached Bytecode (.jsc / .bin)".as_ptr(),
    extensions: [c"jsc".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VAG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vag".as_ptr(),
    mime_type: c"audio/x-vag".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony PlayStation SPU VAG Audio (.vag)".as_ptr(),
    extensions: [c"vag".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VALVE_ANI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"valve_ani".as_ptr(),
    mime_type: c"model/x-valve-ani".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Valve Source Engine StudioMDL External Animation Data (.ani, 2004) Valve".as_ptr(),
    extensions: [c"ani".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VALVE_HAMMER_RMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"valve_hammer_rmf".as_ptr(),
    mime_type: c"application/x-valve-rmf".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Valve Hammer Editor / Worldcraft Rich Map Format (.rmf, 1996) Valve".as_ptr(),
    extensions: [c"rmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VALVE_MDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"valve_mdl".as_ptr(),
    mime_type: c"model/x-goldsrc-mdl".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Valve GoldSrc / Source Studio 3D Model (.mdl)".as_ptr(),
    extensions: [c"mdl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VALVE_SHADER_VCS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"valve_shader_vcs".as_ptr(),
    mime_type: c"application/x-valve-shader-vcs".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Valve Source Engine LZMA-Compiled Shader (.vcs, v6)".as_ptr(),
    extensions: [c"vcs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VALVE_TOOLS_ASSET_INFO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"valve_tools_asset_info".as_ptr(),
    mime_type: c"application/x-valve-tools-asset-info".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Valve Source 2 Tools Asset Info Index (tools_asset_info.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VAM_VAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vam_vab".as_ptr(),
    mime_type: c"application/x-vam-vab".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Virt-A-Mate DynamicStore Binary Mesh (.vab, 2018) MeshedVR".as_ptr(),
    extensions: [c"vab".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VBA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vba".as_ptr(),
    mime_type: c"text/vbscript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"MS Visual Basic source (VBA)".as_ptr(),
    extensions: [c"vbs".as_ptr(), c"vba".as_ptr(), c"vb".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VBE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vbe".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"code".as_ptr(),
    description: c"EncryptedVBS".as_ptr(),
    extensions: [c"vbe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VCARD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vcard".as_ptr(),
    mime_type: c"text/vcard".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"vcard".as_ptr(),
    extensions: [c"vcard".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VCF_GENOMICS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vcf_genomics".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Variant Call Format".as_ptr(),
    extensions: [c"vcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VCS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vcs".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"vcs".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VCTR_VECTOR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vctr_vector".as_ptr(),
    mime_type: c"application/x-vctr-vector".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"vctr Binary-Framed JSON Vector (.vctr)".as_ptr(),
    extensions: [c"vctr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VCXPROJ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vcxproj".as_ptr(),
    mime_type: c"application/xml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Visual Studio MSBuild project".as_ptr(),
    extensions: [c"vcxproj".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VDEX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vdex".as_ptr(),
    mime_type: c"application/vnd.android.vdex".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Android Verifier DEX (VDEX)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VECTOR_BLF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vector_blf".as_ptr(),
    mime_type: c"application/x-vector-blf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Vector Informatik Binary Logging Format CAN/LIN/FlexRay Bus Log (.blf)".as_ptr(),
    extensions: [c"blf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VECTOR_FWK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vector_fwk".as_ptr(),
    mime_type: c"application/x-vector-fwk".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"VecTor Panel PV19 Finite Element Workspace (.fwk)".as_ptr(),
    extensions: [c"fwk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VERILOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"verilog".as_ptr(),
    mime_type: c"text/x-verilog".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Verilog source".as_ptr(),
    extensions: [c"v".as_ptr(), c"verilog".as_ptr(), c"vlg".as_ptr(), c"vh".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VFB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vfb".as_ptr(),
    mime_type: c"application/x-fontlab-vfb".as_ptr(),
    group: c"font".as_ptr(),
    description: c"FontLab Studio Vector Font Binary (.vfb)".as_ptr(),
    extensions: [c"vfb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VFD_FDD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vfd_fdd".as_ptr(),
    mime_type: c"application/x-pc98-fdd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"NEC PC-98 Virtual98 Floppy Disk Image (.fdd)".as_ptr(),
    extensions: [c"fdd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VGM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vgm".as_ptr(),
    mime_type: c"audio/x-vgm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Video Game Music Sound Chip Log (.vgm)".as_ptr(),
    extensions: [c"vgm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VHD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vhd".as_ptr(),
    mime_type: c"application/x-vhd".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Virtual Hard Disk".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VHDL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vhdl".as_ptr(),
    mime_type: c"text/x-vhdl".as_ptr(),
    group: c"code".as_ptr(),
    description: c"VHDL source".as_ptr(),
    extensions: [c"vhd".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VHDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vhdx".as_ptr(),
    mime_type: c"application/x-vhdx".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Hyper-V Virtual Hard Disk v2 (.vhdx)".as_ptr(),
    extensions: [c"vhdx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vib".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"VMware Installation Bundle".as_ptr(),
    extensions: [c"vib".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIC20_ROM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vic20_rom".as_ptr(),
    mime_type: c"application/x-vic20-rom".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Commodore VIC-20 Cartridge ROM / PCVIC Snapshot (.prg / .crt / .20 / .a0)".as_ptr(),
    extensions: [c"prg".as_ptr(), c"crt".as_ptr(), c"a0".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIC2_HUFFMAN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vic2_huffman".as_ptr(),
    mime_type: c"application/x-atari-vic2".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Atari ST Vic2 Huffman Compressed Data (Vic2)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIM_SPELL_SPL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vim_spell_spl".as_ptr(),
    mime_type: c"application/x-vim-spell-spl".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Vim Compiled Spell Checking Dictionary (.spl, 2006) Bram Moolenaar".as_ptr(),
    extensions: [c"spl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIM_SWAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vim_swap".as_ptr(),
    mime_type: c"application/x-vim-swap".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Vim Swap File (.swp)".as_ptr(),
    extensions: [c"swp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIO2_SCORE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vio2_score".as_ptr(),
    mime_type: c"application/x-vio2-score".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"VIO v2 Music Score (.vio)".as_ptr(),
    extensions: [c"vio".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIPS_IMAGE_V: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vips_image_v".as_ptr(),
    mime_type: c"application/x-vips-image-v".as_ptr(),
    group: c"document".as_ptr(),
    description: c"libvips / VIPS Native Image (.v / .vips, PRONOM fmt/1811)".as_ptr(),
    extensions: [c"v".as_ptr(), c"vips".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VIRTUALDJ_VDJSAMPLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"virtualdj_vdjsample".as_ptr(),
    mime_type: c"application/x-virtualdj-vdjsample".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Atomix VirtualDJ Audio/Video Sampler Clip (.vdjsample, 2014) Atomix Productions".as_ptr(),
    extensions: [c"vdjsample".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VISIO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"visio".as_ptr(),
    mime_type: c"application/vnd.ms-visio.drawing.main+xml".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Visio".as_ptr(),
    extensions: [c"vsd".as_ptr(), c"vsdm".as_ptr(), c"vsdx".as_ptr(), c"vdw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VMDK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vmdk".as_ptr(),
    mime_type: c"application/x-raw-disk-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"VMware Virtual Disk Image (.vmdk)".as_ptr(),
    extensions: [c"vmdk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VMS_EXE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vms_exe".as_ptr(),
    mime_type: c"application/x-vms-executable".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"DEC VAX/VMS Executable / Shareable Image (.exe, 1977) Digital Equipment".as_ptr(),
    extensions: [c"exe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VMS_LIB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vms_lib".as_ptr(),
    mime_type: c"application/x-vms-library".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DEC VAX/VMS Librarian Archive (.olb / .mlb / .hlb / .nlb / .tlb, 1978) Digital Equipment".as_ptr(),
    extensions: [c"olb".as_ptr(), c"mlb".as_ptr(), c"hlb".as_ptr(), c"nlb".as_ptr(), c"tlb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VMWARE_NVRAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vmware_nvram".as_ptr(),
    mime_type: c"application/x-vmware-nvram".as_ptr(),
    group: c"application".as_ptr(),
    description: c"VMware virtual machine NVRAM".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VOODOO_VSS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"voodoo_vss".as_ptr(),
    mime_type: c"audio/x-voodoo-vss".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Thomas Winischhofer Voodoo Supreme Synthesizer Module (.vss)".as_ptr(),
    extensions: [c"vss".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VOXLAP_KV6: MagikaTypeInfo = MagikaTypeInfo {
    label: c"voxlap_kv6".as_ptr(),
    mime_type: c"application/x-voxlap-kv6".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Ken Silverman Voxlap Engine KV6 Voxel Sprite (.kv6, 2000) Ken Silverman".as_ptr(),
    extensions: [c"kv6".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VOYETRA_PATCH_BANK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"voyetra_patch_bank".as_ptr(),
    mime_type: c"audio/x-voyetra-bank".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Voyetra PatchMaster / Sideman MIDI Synthesizer Bank (.b*, .c*, 1987) Voyetra Technologies".as_ptr(),
    extensions: [c"b".as_ptr(), c"c".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vpk".as_ptr(),
    mime_type: c"application/x-vpk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Valve Pak Source Engine Archive (.vpk)".as_ptr(),
    extensions: [c"vpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VST2_FXP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vst2_fxp".as_ptr(),
    mime_type: c"application/x-vst2-preset".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Steinberg VST 2 Plug-In Preset / Bank (.fxp / .fxb)".as_ptr(),
    extensions: [c"fxp".as_ptr(), c"fxb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VST3_VSTPRESET: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vst3_vstpreset".as_ptr(),
    mime_type: c"application/x-vst3-preset".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Steinberg VST 3 Plug-In Preset (.vstpreset)".as_ptr(),
    extensions: [c"vstpreset".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VTECH_VRP_LOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vtech_vrp_log".as_ptr(),
    mime_type: c"application/x-vtech-vrp-log".as_ptr(),
    group: c"application".as_ptr(),
    description: c"VTech Multi-Section Install Log Container (.vrp)".as_ptr(),
    extensions: [c"vrp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VTF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vtf".as_ptr(),
    mime_type: c"image/x-vtf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Valve Source Engine Texture Format (.vtf)".as_ptr(),
    extensions: [c"vtf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VTK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vtk".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"VTK dataset".as_ptr(),
    extensions: [c"vtk".as_ptr(), c"vtp".as_ptr(), c"vtu".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VTT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vtt".as_ptr(),
    mime_type: c"text/vtt".as_ptr(),
    group: c"text".as_ptr(),
    description: c"Web Video Text Tracks".as_ptr(),
    extensions: [c"vtt".as_ptr(), c"webvtt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VUE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vue".as_ptr(),
    mime_type: c"application/javascript".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Vue source".as_ptr(),
    extensions: [c"vue".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static VVD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vvd".as_ptr(),
    mime_type: c"model/x-valve-vvd".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Valve Source Engine StudioMDL Vertex Data (.vvd)".as_ptr(),
    extensions: [c"vvd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static VW_SGO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"vw_sgo".as_ptr(),
    mime_type: c"application/x-vw-sgo".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Volkswagen / Audi ODIS / VAS-PC SGML Object ECU Flash Container (.sgo) Volkswagen".as_ptr(),
    extensions: [c"sgo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static W64: MagikaTypeInfo = MagikaTypeInfo {
    label: c"w64".as_ptr(),
    mime_type: c"audio/x-w64".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony Wave64 64-bit audio container (.w64)".as_ptr(),
    extensions: [c"w64".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WAD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wad".as_ptr(),
    mime_type: c"application/wad".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"WAD".as_ptr(),
    extensions: [c"wad".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WAMR_AOT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wamr_aot".as_ptr(),
    mime_type: c"application/x-wamr-aot".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Bytecode Alliance WebAssembly Micro Runtime AOT Module (.aot)".as_ptr(),
    extensions: [c"aot".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WARCRAFT3_MAP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"warcraft3_map".as_ptr(),
    mime_type: c"application/x-warcraft3-map".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Blizzard Entertainment Warcraft III World Editor Map (.w3m / .w3x, 2002) Blizzard Entertainment".as_ptr(),
    extensions: [c"w3m".as_ptr(), c"w3x".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WARCRAFT3_MDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"warcraft3_mdx".as_ptr(),
    mime_type: c"model/x-warcraft3-mdx".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Blizzard Entertainment Warcraft III Binary 3D Model (.mdx, 2002) Blizzard Entertainment".as_ptr(),
    extensions: [c"mdx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WARCRAFT3_REPLAY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"warcraft3_replay".as_ptr(),
    mime_type: c"application/x-warcraft3-replay".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Blizzard Entertainment Warcraft III Recorded Game Replay (.w3g, 2002) Blizzard Entertainment".as_ptr(),
    extensions: [c"w3g".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WARFRAME_TOC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"warframe_toc".as_ptr(),
    mime_type: c"application/x-warframe-toc".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Digital Extremes Evolution Engine (Warframe) Cache TOC Index (.toc)".as_ptr(),
    extensions: [c"toc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WASM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wasm".as_ptr(),
    mime_type: c"application/wasm".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Web Assembly".as_ptr(),
    extensions: [c"wasm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WATCOM_WPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"watcom_wpk".as_ptr(),
    mime_type: c"application/x-watcom-wpk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Watcom Installer WPK Archive (.wpk)".as_ptr(),
    extensions: [c"wpk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wav".as_ptr(),
    mime_type: c"audio/x-wav".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Waveform Audio file (WAV)".as_ptr(),
    extensions: [c"wav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WBN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wbn".as_ptr(),
    mime_type: c"application/webbundle".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"W3C / Google Chrome WebBundle & Isolated Web App (.wbn / .swbn)".as_ptr(),
    extensions: [c"wbn".as_ptr(), c"swbn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WCX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wcx".as_ptr(),
    mime_type: c"text/x-wildcat-wcx".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Mustang Wildcat! WCX BBS screen (.wcx)".as_ptr(),
    extensions: [c"wcx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WEBM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"webm".as_ptr(),
    mime_type: c"video/webm".as_ptr(),
    group: c"video".as_ptr(),
    description: c"WebM media file".as_ptr(),
    extensions: [c"webm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WEBP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"webp".as_ptr(),
    mime_type: c"image/webp".as_ptr(),
    group: c"image".as_ptr(),
    description: c"WebP media file".as_ptr(),
    extensions: [c"webp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WELTENBAUER_WBP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"weltenbauer_wbp".as_ptr(),
    mime_type: c"application/x-weltenbauer-wbp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Weltenbauer / Astragon Construction Simulator Persistence Save (.wbp)".as_ptr(),
    extensions: [c"wbp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WESTWOOD_CSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"westwood_csf".as_ptr(),
    mime_type: c"application/x-westwood-csf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Westwood Studios / EA SAGE Engine Compiled String Table (.csf, 2000) Westwood Studios".as_ptr(),
    extensions: [c"csf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WESTWOOD_VXL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"westwood_vxl".as_ptr(),
    mime_type: c"model/x-westwood-vxl".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Westwood Studios Command & Conquer Voxel Animation (.vxl, 1999) Westwood Studios".as_ptr(),
    extensions: [c"vxl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WGT_FONT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wgt_font".as_ptr(),
    mime_type: c"application/x-wgt-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"WordUp Graphics Toolkit Bitmap Font (.wfn)".as_ptr(),
    extensions: [c"wfn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WHATSAPP_CRYPT14: MagikaTypeInfo = MagikaTypeInfo {
    label: c"whatsapp_crypt14".as_ptr(),
    mime_type: c"application/x-whatsapp-crypt14".as_ptr(),
    group: c"application".as_ptr(),
    description: c"WhatsApp Encrypted Msgstore Database Backup (.crypt14, 2021) WhatsApp".as_ptr(),
    extensions: [c"crypt14".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WHITEWATER_ACTOR_IMA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"whitewater_actor_ima".as_ptr(),
    mime_type: c"application/x-whitewater-actor-image".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Whitewater Group Actor Object-Oriented Image (.ima, 1987) Whitewater Group".as_ptr(),
    extensions: [c"ima".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WII_GECKO_GCT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wii_gecko_gct".as_ptr(),
    mime_type: c"application/x-wii-gecko-gct".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Nintendo Wii Gecko / Ocarina Cheat Code Table (.gct)".as_ptr(),
    extensions: [c"gct".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wim".as_ptr(),
    mime_type: c"application/x-ms-wim".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"Windows Imaging Format".as_ptr(),
    extensions: [c"wim".as_ptr(), c"swm".as_ptr(), c"esd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN31_REG_DAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win31_reg_dat".as_ptr(),
    mime_type: c"application/x-win31-registry".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows 3.1 Registration Info Database (REG.DAT, SHCC3.10)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN_CAL: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win_cal".as_ptr(),
    mime_type: c"application/x-ms-calendar".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Windows 1.x-3.1 Calendar (.cal)".as_ptr(),
    extensions: [c"cal".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN_GRP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win_grp".as_ptr(),
    mime_type: c"application/x-ms-grp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows 3.x Program Manager Group (.grp)".as_ptr(),
    extensions: [c"grp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN_KDUMP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win_kdump".as_ptr(),
    mime_type: c"application/x-windows-dmp".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Kernel Memory Crash Dump (PAGEDUMP / PAGEDU64)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN_RESTORE_RP_LOG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win_restore_rp_log".as_ptr(),
    mime_type: c"application/x-ms-restore-point-log".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows System Restore Point Log (rp.log)".as_ptr(),
    extensions: [c"log".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN_SUPERFETCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win_superfetch".as_ptr(),
    mime_type: c"application/x-ms-superfetch".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows SuperFetch / ReadyBoot Trace Database (AgGl*.db)".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIN_TASK_JOB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"win_task_job".as_ptr(),
    mime_type: c"application/x-win-task-job".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Task Scheduler Job Object (.job, 1997) Microsoft".as_ptr(),
    extensions: [c"job".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINBOX_CDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"winbox_cdb".as_ptr(),
    mime_type: c"application/x-mikrotik-winbox-cdb".as_ptr(),
    group: c"database".as_ptr(),
    description: c"MikroTik WinBox Address Book & RouterOS Configuration Database (.cdb, 2002) MikroTik".as_ptr(),
    extensions: [c"cdb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINCE_B000FF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wince_b000ff".as_ptr(),
    mime_type: c"application/x-wince-rom-image".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows CE ROM Binary Image Data Format (.nb0 / .bin)".as_ptr(),
    extensions: [c"nb0".as_ptr(), c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINCE_MSCE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wince_msce".as_ptr(),
    mime_type: c"application/x-wince-cab".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft Windows CE Installation Cabinet Header (MSCE)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_ADDRESS_BOOK_WAB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_address_book_wab".as_ptr(),
    mime_type: c"application/x-windows-address-book-wab".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Windows Address Book (.wab, PRONOM fmt/1255)".as_ptr(),
    extensions: [c"wab".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_CACHES_DB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_caches_db".as_ptr(),
    mime_type: c"application/x-windows-caches-db".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Microsoft Windows Property / App Cache Database (.db)".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_CDF_MS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_cdf_ms".as_ptr(),
    mime_type: c"application/x-windows-cdf-ms".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Component-Based Servicing Compiled Form (.cdf-ms)".as_ptr(),
    extensions: [c"cdf-ms".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_CODEPAGE_NLS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_codepage_nls".as_ptr(),
    mime_type: c"application/x-windows-codepage-nls".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows National Language Support Codepage Table (C_*.NLS, 1993) Microsoft".as_ptr(),
    extensions: [c"nls".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_IME_MB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_ime_mb".as_ptr(),
    mime_type: c"application/x-windows-ime-mb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Input Method Editor Compiled Dictionary Table (.mb)".as_ptr(),
    extensions: [c"mb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_INSTALLER_SCRIPT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_installer_script".as_ptr(),
    mime_type: c"application/x-msi-script".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Installer Binary Execution / Rollback Script (.ibs / .rbs)".as_ptr(),
    extensions: [c"ibs".as_ptr(), c"rbs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_PNF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_pnf".as_ptr(),
    mime_type: c"application/x-windows-pnf".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Precompiled Setup Information (.pnf, 2001) Microsoft".as_ptr(),
    extensions: [c"pnf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_PRI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_pri".as_ptr(),
    mime_type: c"application/x-ms-pri".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows Package Resource Index (.pri)".as_ptr(),
    extensions: [c"pri".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINDOWS_SDI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"windows_sdi".as_ptr(),
    mime_type: c"application/x-ms-sdi".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Microsoft Windows System Deployment Image (.sdi, $SDI0001)".as_ptr(),
    extensions: [c"sdi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINK_WNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wink_wnk".as_ptr(),
    mime_type: c"application/x-wink".as_ptr(),
    group: c"video".as_ptr(),
    description: c"DebugMode Wink Screen Capture Presentation (.wnk)".as_ptr(),
    extensions: [c"wnk".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINK_WTP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wink_wtp".as_ptr(),
    mime_type: c"application/x-wink-template".as_ptr(),
    group: c"video".as_ptr(),
    description: c"DebugMode Wink Presentation Template (.wtp)".as_ptr(),
    extensions: [c"wtp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINOLS_KP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"winols_kp".as_ptr(),
    mime_type: c"application/x-winols-kp".as_ptr(),
    group: c"document".as_ptr(),
    description: c"EVC Electronic WinOLS ECU Map Pack (.kp) EVC electronic".as_ptr(),
    extensions: [c"kp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WINREGISTRY: MagikaTypeInfo = MagikaTypeInfo {
    label: c"winregistry".as_ptr(),
    mime_type: c"text/x-ms-regedit".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Windows Registry text".as_ptr(),
    extensions: [c"reg".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static WIREDTIGER: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wiredtiger".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"WiredTiger storage".as_ptr(),
    extensions: [c"wt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIRELESS_REGDB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wireless_regdb".as_ptr(),
    mime_type: c"application/x-wireless-regdb".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Linux CRDA / cfg80211 Wireless Regulatory Database (regulatory.db, RGDB)".as_ptr(),
    extensions: [c"db".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WIRESYNC_MOMV2_OMO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wiresync_momv2_omo".as_ptr(),
    mime_type: c"application/x-wiresync-momv2-omo".as_ptr(),
    group: c"application".as_ptr(),
    description: c"WireSyncEngine Obfuscated Model (.omo)".as_ptr(),
    extensions: [c"omo".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WITCHER2_DZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"witcher2_dzip".as_ptr(),
    mime_type: c"application/x-witcher2-dzip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CD Projekt RED REDengine 2 / The Witcher 2 DZIP Archive (.dzip, 2011) CD Projekt RED".as_ptr(),
    extensions: [c"dzip".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WITCHER3_BUNDLE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"witcher3_bundle".as_ptr(),
    mime_type: c"application/x-witcher3-bundle".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"CD Projekt RED REDengine 3 / The Witcher 3 Asset Bundle (.bundle, 2015) CD Projekt RED".as_ptr(),
    extensions: [c"bundle".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WITCHER3_SAV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"witcher3_sav".as_ptr(),
    mime_type: c"application/x-witcher3-sav".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CD Projekt RED The Witcher 3 Saved Game (.sav, 2015) CD Projekt RED".as_ptr(),
    extensions: [c"sav".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WITCHER3_W3SPEECH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"witcher3_w3speech".as_ptr(),
    mime_type: c"application/x-witcher3-w3speech".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"CD Projekt RED REDengine 3 / The Witcher 3 Speech Container (.w3speech / .meta, 2015) CD Projekt RED".as_ptr(),
    extensions: [c"w3speech".as_ptr(), c"meta".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WITCHER3_W3STRINGS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"witcher3_w3strings".as_ptr(),
    mime_type: c"application/x-witcher3-w3strings".as_ptr(),
    group: c"database".as_ptr(),
    description: c"CD Projekt RED REDengine 3 / The Witcher 3 Localization Strings (.w3strings, 2015) CD Projekt RED".as_ptr(),
    extensions: [c"w3strings".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WITCHER_CR2W: MagikaTypeInfo = MagikaTypeInfo {
    label: c"witcher_cr2w".as_ptr(),
    mime_type: c"application/x-redengine-cr2w".as_ptr(),
    group: c"application".as_ptr(),
    description: c"CD Projekt RED REDengine CR2W Resource (.w2ent / .w2mesh / .w2scene / .xbm, 2011) CD Projekt RED".as_ptr(),
    extensions: [c"w2ent".as_ptr(), c"w2mesh".as_ptr(), c"w2scene".as_ptr(), c"xbm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WKT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wkt".as_ptr(),
    mime_type: c"text/plain".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Well-Known Text geometry".as_ptr(),
    extensions: [c"wkt".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static WMA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wma".as_ptr(),
    mime_type: c"audio/x-ms-wma".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Windows Media Audio".as_ptr(),
    extensions: [c"wma".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wmf".as_ptr(),
    mime_type: c"image/wmf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Windows metafile".as_ptr(),
    extensions: [c"wmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WMV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wmv".as_ptr(),
    mime_type: c"video/x-ms-wmv".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Windows Media Video".as_ptr(),
    extensions: [c"wmv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"woff".as_ptr(),
    mime_type: c"font/woff".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Web Open Font Format".as_ptr(),
    extensions: [c"woff".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOFF2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"woff2".as_ptr(),
    mime_type: c"font/woff2".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Web Open Font Format v2".as_ptr(),
    extensions: [c"woff2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOLFRPG_MAP_MPS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wolfrpg_map_mps".as_ptr(),
    mime_type: c"application/x-wolfrpg-map-mps".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"WOLF RPG Editor Map File (.mps)".as_ptr(),
    extensions: [c"mps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOODWORKS_CWB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"woodworks_cwb".as_ptr(),
    mime_type: c"application/x-woodworks-cwb".as_ptr(),
    group: c"document".as_ptr(),
    description: c"WoodWorks Sizer / Connections Project (.cwb) Canadian Wood Council".as_ptr(),
    extensions: [c"cwb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WORDSTAR_2000: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wordstar_2000".as_ptr(),
    mime_type: c"application/x-wordstar2000".as_ptr(),
    group: c"document".as_ptr(),
    description: c"MicroPro WordStar 2000 Document (.ws2)".as_ptr(),
    extensions: [c"ws2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WORLDCRAFT_CMDSEQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"worldcraft_cmdseq".as_ptr(),
    mime_type: c"application/x-worldcraft-cmdseq".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Valve Hammer / Worldcraft Compile Command Sequences (CmdSeq.wc, 1997) Valve".as_ptr(),
    extensions: [c"wc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOW_DBC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wow_dbc".as_ptr(),
    mime_type: c"application/x-wow-dbc".as_ptr(),
    group: c"database".as_ptr(),
    description: c"Blizzard World of Warcraft Client Database (.dbc / .db2, 2004) Blizzard Entertainment".as_ptr(),
    extensions: [c"dbc".as_ptr(), c"db2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOW_M2: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wow_m2".as_ptr(),
    mime_type: c"model/x-wow-m2".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Blizzard World of Warcraft M2 3D Model (.m2, 2004) Blizzard Entertainment".as_ptr(),
    extensions: [c"m2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WOZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"woz".as_ptr(),
    mime_type: c"application/x-floppy-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Applesauce WOZ floppy bitstream image".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WPS_DOS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wps_dos".as_ptr(),
    mime_type: c"application/vnd.ms-works".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Works for MS-DOS Word Processor Document (.wps)".as_ptr(),
    extensions: [c"wps".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WPSPLUS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wpsplus".as_ptr(),
    mime_type: c"application/x-dec-wpsplus".as_ptr(),
    group: c"document".as_ptr(),
    description: c"DEC WPS-Plus Word Processing Document (.wps / .wpl / .dx)".as_ptr(),
    extensions: [c"wps".as_ptr(), c"wpl".as_ptr(), c"dx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WRITEON_WOF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"writeon_wof".as_ptr(),
    mime_type: c"font/x-hercules-wof".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Hercules Write On! RamFont Bitmap Font (.wof, 1987) Hercules Computer Technology".as_ptr(),
    extensions: [c"wof".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WSQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wsq".as_ptr(),
    mime_type: c"image/x-wsq".as_ptr(),
    group: c"image".as_ptr(),
    description: c"FBI Wavelet Scalar Quantization Grayscale Fingerprint Image (.wsq)".as_ptr(),
    extensions: [c"wsq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WTV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wtv".as_ptr(),
    mime_type: c"video/x-ms-wtv".as_ptr(),
    group: c"video".as_ptr(),
    description: c"Windows Media Center Recorded TV Show (.wtv)".as_ptr(),
    extensions: [c"wtv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static WWISE_BNK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"wwise_bnk".as_ptr(),
    mime_type: c"audio/x-wwise-bnk".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Audiokinetic Wwise SoundBank / File Package (.bnk / .pck)".as_ptr(),
    extensions: [c"bnk".as_ptr(), c"pck".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static X64DBG_TRACE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"x64dbg_trace".as_ptr(),
    mime_type: c"application/x-x64dbg-trace".as_ptr(),
    group: c"application".as_ptr(),
    description: c"x64dbg Binary Execution Trace (.trace32 / .trace64)".as_ptr(),
    extensions: [c"trace32".as_ptr(), c"trace64".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static X68000_MDX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"x68000_mdx".as_ptr(),
    mime_type: c"audio/x-mdx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sharp X68000 MXDRV Music Module (.mdx)".as_ptr(),
    extensions: [c"mdx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XACT_XGS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xact_xgs".as_ptr(),
    mime_type: c"application/x-ms-xact-xgs".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Microsoft XACT Global Settings (.xgs)".as_ptr(),
    extensions: [c"xgs".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XACT_XSB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xact_xsb".as_ptr(),
    mime_type: c"audio/x-ms-xact-xsb".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Microsoft XACT Sound Bank (.xsb)".as_ptr(),
    extensions: [c"xsb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XACT_XWB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xact_xwb".as_ptr(),
    mime_type: c"audio/x-ms-xwb".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Microsoft DirectX XACT Wave Bank (.xwb)".as_ptr(),
    extensions: [c"xwb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XALZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xalz".as_ptr(),
    mime_type: c"application/x-xamarin-xalz".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Xamarin.Android / .NET for Android LZ4-Compressed Assembly (XALZ)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XAR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xar".as_ptr(),
    mime_type: c"application/x-xar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"XAR archive compressed data".as_ptr(),
    extensions: [c"pkg".as_ptr(), c"xar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XAR_IMAGE_FORMAT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xar_image_format".as_ptr(),
    mime_type: c"application/x-xar-image-format".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Xara X / Xara Extreme Vector Drawing (.xar, PRONOM fmt/922)".as_ptr(),
    extensions: [c"xar".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xbin".as_ptr(),
    mime_type: c"image/x-xbin".as_ptr(),
    group: c"image".as_ptr(),
    description: c"XBIN Extended Binary Text-Mode Image (.xb)".as_ptr(),
    extensions: [c"xb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XBOX_BDES_XZP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xbox_bdes_xzp".as_ptr(),
    mime_type: c"application/x-xbox-bdes-xzp".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"BDES Encrypted Package (.xzp)".as_ptr(),
    extensions: [c"xzp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XBOX_SDNS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xbox_sdns".as_ptr(),
    mime_type: c"audio/x-xbox-sdns".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Xbox SDNS ADPCM Audio Stream (.sdns)".as_ptr(),
    extensions: [c"sdns".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XBOX_XBE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xbox_xbe".as_ptr(),
    mime_type: c"application/x-xbox-executable".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"Microsoft Original Xbox Executable (.xbe)".as_ptr(),
    extensions: [c"xbe".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XCAR_FNT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xcar_fnt".as_ptr(),
    mime_type: c"application/x-font".as_ptr(),
    group: c"font".as_ptr(),
    description: c"Bethesda Softworks XCar Bitmap Font (FNHD .fnt)".as_ptr(),
    extensions: [c"fnt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XCF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xcf".as_ptr(),
    mime_type: c"image/x-xcf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Gimp image".as_ptr(),
    extensions: [c"xcf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XCOFF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xcoff".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"executable".as_ptr(),
    description: c"XCOFF".as_ptr(),
    extensions: [c"o".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XCR_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xcr_archive".as_ptr(),
    mime_type: c"application/x-xcr-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"XCR Archive (.xcr)".as_ptr(),
    extensions: [c"xcr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XFS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xfs".as_ptr(),
    mime_type: c"application/x-raw-disk-image".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Silicon Graphics XFS High-Performance Filesystem".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XIAOMI_HDR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xiaomi_hdr".as_ptr(),
    mime_type: c"application/x-xiaomi-hdr".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Xiaomi MiWiFi HDR1 / HDR2 Router Firmware Image (.bin)".as_ptr(),
    extensions: [c"bin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XILAM_DERF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xilam_derf".as_ptr(),
    mime_type: c"audio/x-derf".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Xilam / Stupid Invaders DERF DPCM Audio (.adp)".as_ptr(),
    extensions: [c"adp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XILINX_BIT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xilinx_bit".as_ptr(),
    mime_type: c"application/x-xilinx-bitstream".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Xilinx FPGA Configuration Bitstream (.bit)".as_ptr(),
    extensions: [c"bit".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XILINX_XCLBIN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xilinx_xclbin".as_ptr(),
    mime_type: c"application/x-xilinx-xclbin".as_ptr(),
    group: c"application".as_ptr(),
    description: c"AMD / Xilinx XRT FPGA Accelerator Binary Container (.xclbin, xclbin2\\0)".as_ptr(),
    extensions: [c"xclbin".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XL_PAINT_MAX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xl_paint_max".as_ptr(),
    mime_type: c"application/x-xl-paint-max".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Atari 8-Bit XL-Paint MAX Interlaced Image (.max, PRONOM fmt/1658)".as_ptr(),
    extensions: [c"max".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XL_PAINT_RAW: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xl_paint_raw".as_ptr(),
    mime_type: c"application/x-xl-paint-raw".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Atari 8-Bit XL-Paint Raw Image (.raw, PRONOM fmt/1659)".as_ptr(),
    extensions: [c"raw".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XLS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xls".as_ptr(),
    mime_type: c"application/vnd.ms-excel".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Excel CDF document".as_ptr(),
    extensions: [c"xls".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XLSB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xlsb".as_ptr(),
    mime_type: c"application/vnd.ms-excel.sheet.binary.macroEnabled.12".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Excel 2007+ document (binary format)".as_ptr(),
    extensions: [c"xlsb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XLSX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xlsx".as_ptr(),
    mime_type: c"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Microsoft Excel 2007+ document".as_ptr(),
    extensions: [c"xlsx".as_ptr(), c"xlsm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xm".as_ptr(),
    mime_type: c"audio/x-mod".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"FastTracker II Extended Module Audio (.xm)".as_ptr(),
    extensions: [c"xm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xml".as_ptr(),
    mime_type: c"text/xml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"XML document".as_ptr(),
    extensions: [c"xml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static XNA_COURSE_CRS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xna_course_crs".as_ptr(),
    mime_type: c"application/x-xna-course-crs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Windows Phone XNA 711x7.0 Course Package (.crs_)".as_ptr(),
    extensions: [c"crs_".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XNB: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xnb".as_ptr(),
    mime_type: c"application/x-xna-xnb".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Microsoft XNA Game Studio / MonoGame Compiled Binary Asset (.xnb)".as_ptr(),
    extensions: [c"xnb".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XP3: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xp3".as_ptr(),
    mime_type: c"application/x-kirikiri-xp3".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"KiriKiri2 / KiriKiriZ Visual Novel Archive (.xp3)".as_ptr(),
    extensions: [c"xp3".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XPI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xpi".as_ptr(),
    mime_type: c"application/zip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Compressed installation archive (XPI)".as_ptr(),
    extensions: [c"xpi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XPK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xpk".as_ptr(),
    mime_type: c"application/x-xpk".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Amiga eXternal Package (XPK) Compressed File".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XPLANE_ACF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xplane_acf".as_ptr(),
    mime_type: c"application/x-xplane-acf".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Laminar Research X-Plane Binary Aircraft Model (.acf, v7)".as_ptr(),
    extensions: [c"acf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XPLANE_DSF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xplane_dsf".as_ptr(),
    mime_type: c"model/x-xplane-dsf".as_ptr(),
    group: c"geometry".as_ptr(),
    description: c"Laminar Research X-Plane Distribution Scenery Format (.dsf)".as_ptr(),
    extensions: [c"dsf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XPLAPP_PACKAGE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xplapp_package".as_ptr(),
    mime_type: c"application/x-xplapp-package".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"XPLAPP Application Package (.xplapp)".as_ptr(),
    extensions: [c"xplapp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XSD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xsd".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"unknown".as_ptr(),
    description: c"xsd".as_ptr(),
    extensions: [c"xsd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XSM_MUSIC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xsm_music".as_ptr(),
    mime_type: c"audio/x-xsm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Davey Taylor eXtra Simple Music AdLib Module (.xsm, ofTAZ!)".as_ptr(),
    extensions: [c"xsm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XV_THUMBNAIL_P7: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xv_thumbnail_p7".as_ptr(),
    mime_type: c"application/x-xv-thumbnail-p7".as_ptr(),
    group: c"document".as_ptr(),
    description: c"John Bradley XV Visual Schnauzer Thumbnail (.p7, PRONOM fmt/1497)".as_ptr(),
    extensions: [c"p7".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XVAG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xvag".as_ptr(),
    mime_type: c"audio/x-xvag".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Sony PlayStation 3 / PS Vita / PS4 XVAG Audio Stream (.xvag)".as_ptr(),
    extensions: [c"xvag".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static XZ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"xz".as_ptr(),
    mime_type: c"application/x-xz".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"XZ compressed data".as_ptr(),
    extensions: [c"xz".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YAML: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yaml".as_ptr(),
    mime_type: c"application/x-yaml".as_ptr(),
    group: c"code".as_ptr(),
    description: c"YAML source".as_ptr(),
    extensions: [c"yml".as_ptr(), c"yaml".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static YARA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yara".as_ptr(),
    mime_type: c"text/x-yara".as_ptr(),
    group: c"code".as_ptr(),
    description: c"YARA rule".as_ptr(),
    extensions: [c"yar".as_ptr(), c"yara".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static YAY0: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yay0".as_ptr(),
    mime_type: c"application/x-yay0".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Yay0 SZP Compressed Stream (.yay0 / .szp)".as_ptr(),
    extensions: [c"yay0".as_ptr(), c"szp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YAZ0: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yaz0".as_ptr(),
    mime_type: c"application/x-yaz0".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Nintendo Yaz0 Run-Length Compressed Stream (.szs / .yaz0)".as_ptr(),
    extensions: [c"szs".as_ptr(), c"yaz0".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YGLF_FILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yglf_file".as_ptr(),
    mime_type: c"application/x-yglf-file".as_ptr(),
    group: c"image".as_ptr(),
    description: c"yGLF Graphics/Glyph Format (.glf)".as_ptr(),
    extensions: [c"glf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YMST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"ymst".as_ptr(),
    mime_type: c"audio/x-ymst".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"Atari ST YMST YM2149 Chiptune (.ymst)".as_ptr(),
    extensions: [c"ymst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YURIS_YPF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yuris_ypf".as_ptr(),
    mime_type: c"application/x-yuris-ypf".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"YU-RIS Visual Novel Engine YPF Archive (.ypf, 2005) Firstia / YU-RIS".as_ptr(),
    extensions: [c"ypf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static YZ1: MagikaTypeInfo = MagikaTypeInfo {
    label: c"yz1".as_ptr(),
    mime_type: c"application/x-yz1".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"DeepFreezer YZ1 archive (.yz1)".as_ptr(),
    extensions: [c"yz1".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZBR1BGJB_PRN: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zbr1bgjb_prn".as_ptr(),
    mime_type: c"application/x-zbr1bgjb-prn".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Zebra ZBR1BGJB XML-Headered Card Print Job (.prn)".as_ptr(),
    extensions: [c"prn".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZBRUSH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zbrush".as_ptr(),
    mime_type: c"application/octet-stream".as_ptr(),
    group: c"image".as_ptr(),
    description: c"ZBrush project / tool".as_ptr(),
    extensions: [c"zpr".as_ptr(), c"ztl".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZCHUNK_ZCK: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zchunk_zck".as_ptr(),
    mime_type: c"application/zstd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Fedora / RPM zchunk Delta-Compressed File (.zck, \\0ZCK1)".as_ptr(),
    extensions: [c"zck".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZEBRA_TPD: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zebra_tpd".as_ptr(),
    mime_type: c"application/x-zebra-tpd".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Zebra Technologies Printer Driver Profile (.tpd) Zebra Technologies".as_ptr(),
    extensions: [c"tpd".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZEDAL_LIBRARY_LV: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zedal_library_lv".as_ptr(),
    mime_type: c"application/x-zedal-library-lv".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ZEDAL Library Timestamped Binary Export (.lv)".as_ptr(),
    extensions: [c"lv".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZEROPLUS_ALS: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zeroplus_als".as_ptr(),
    mime_type: c"application/x-zeroplus-als".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Zeroplus Logic Analyzer (LAP-C) Project (.als)".as_ptr(),
    extensions: [c"als".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZIG: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zig".as_ptr(),
    mime_type: c"text/zig".as_ptr(),
    group: c"code".as_ptr(),
    description: c"Zig source".as_ptr(),
    extensions: [c"zig".as_ptr(), ptr::null()].as_ptr(),
    is_text: true,
};

#[rustfmt::skip] pub(crate) static ZIGBEE_OTA: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zigbee_ota".as_ptr(),
    mime_type: c"application/x-zigbee-ota".as_ptr(),
    group: c"application".as_ptr(),
    description: c"Zigbee Alliance Cluster Library Over-the-Air Upgrade Image (.ota / .zigbee) Connectivity Standards Alliance".as_ptr(),
    extensions: [c"ota".as_ptr(), c"zigbee".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZIP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zip".as_ptr(),
    mime_type: c"application/zip".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Zip archive data".as_ptr(),
    extensions: [c"zip".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZISOFS_FILE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zisofs_file".as_ptr(),
    mime_type: c"application/x-zisofs".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"H. Peter Anvin zisofs Compressed File (.zf)".as_ptr(),
    extensions: [c"zf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZLIBSTREAM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zlibstream".as_ptr(),
    mime_type: c"application/zlib".as_ptr(),
    group: c"application".as_ptr(),
    description: c"zlib compressed data".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZMF: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zmf".as_ptr(),
    mime_type: c"image/x-zmf".as_ptr(),
    group: c"image".as_ptr(),
    description: c"Zoner Callisto / Zoner Draw Vector Metafile (.zmf)".as_ptr(),
    extensions: [c"zmf".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZOO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zoo".as_ptr(),
    mime_type: c"application/x-zoo".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ZOO compressed archive".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZOOMBROWSER_EX_THUMBNAIL_CACHE_INFO: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zoombrowser_ex_thumbnail_cache_info".as_ptr(),
    mime_type: c"application/x-zoombrowser-ex-thumbnail-cache-info".as_ptr(),
    group: c"document".as_ptr(),
    description: c"Canon ZoomBrowser EX Thumbnail Cache (.info, PRONOM fmt/1496)".as_ptr(),
    extensions: [c"info".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZOS_AR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zos_ar".as_ptr(),
    mime_type: c"application/x-ibm-zos-ar".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"IBM z/OS USS EBCDIC Archiver Library".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZP1_ARCHIVE: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zp1_archive".as_ptr(),
    mime_type: c"application/x-zp1-archive".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"zp1 Compressed Data Package (.zp)".as_ptr(),
    extensions: [c"zp".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZPAQ: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zpaq".as_ptr(),
    mime_type: c"application/x-zpaq".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ZPAQ journaling archive (.zpaq)".as_ptr(),
    extensions: [c"zpaq".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZST: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zst".as_ptr(),
    mime_type: c"application/zstd".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Zstandard".as_ptr(),
    extensions: [c"zst".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZUCCHINI_PATCH: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zucchini_patch".as_ptr(),
    mime_type: c"application/x-zucchini-patch".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"Google Chrome Zucchini Executable Differential Patch (Zuc\\\\0)".as_ptr(),
    extensions: [ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_CHI: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_chi".as_ptr(),
    mime_type: c"audio/x-zx-chi".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Exolon Chip Tracker Module (.chi)".as_ptr(),
    extensions: [c"chi".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_FTC: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_ftc".as_ptr(),
    mime_type: c"audio/x-zx-ftc".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Digital Reality Fast Tracker Module (.ftc)".as_ptr(),
    extensions: [c"ftc".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_FXM: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_fxm".as_ptr(),
    mime_type: c"audio/x-fxm".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Fuxoft AY Music Module (.fxm)".as_ptr(),
    extensions: [c"fxm".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_GTR: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_gtr".as_ptr(),
    mime_type: c"audio/x-zx-gtr".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Polaris Global Tracker Module (.gtr)".as_ptr(),
    extensions: [c"gtr".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_SQT: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_sqt".as_ptr(),
    mime_type: c"audio/x-zx-sqt".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum SQ-Tracker AY Module (.sqt)".as_ptr(),
    extensions: [c"sqt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_ST13: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_st13".as_ptr(),
    mime_type: c"audio/x-zx-st13".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Sound Tracker 1.3 AY Module (.st13)".as_ptr(),
    extensions: [c"st13".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_STP: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_stp".as_ptr(),
    mime_type: c"audio/x-zx-stp".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum KSA Sound Tracker Pro Module (.stp / .stp2)".as_ptr(),
    extensions: [c"stp".as_ptr(), c"stp2".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_TZX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_tzx".as_ptr(),
    mime_type: c"application/x-spectrum-tzx".as_ptr(),
    group: c"archive".as_ptr(),
    description: c"ZX Spectrum / Amstrad CPC Digital Tape Image (.tzx / .cdt)".as_ptr(),
    extensions: [c"tzx".as_ptr(), c"cdt".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZX_VTX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zx_vtx".as_ptr(),
    mime_type: c"audio/x-vtx".as_ptr(),
    group: c"audio".as_ptr(),
    description: c"ZX Spectrum Vortex Project AY/YM Chiptune (.vtx)".as_ptr(),
    extensions: [c"vtx".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

#[rustfmt::skip] pub(crate) static ZYXEL_BIX: MagikaTypeInfo = MagikaTypeInfo {
    label: c"zyxel_bix".as_ptr(),
    mime_type: c"application/x-zyxel-bix".as_ptr(),
    group: c"application".as_ptr(),
    description: c"ZyXEL BIX Router Firmware Image (.bix)".as_ptr(),
    extensions: [c"bix".as_ptr(), ptr::null()].as_ptr(),
    is_text: false,
};

pub(crate) fn content_type_info(content_type: magika::ContentType) -> &'static MagikaTypeInfo {
    match content_type {
        magika::ContentType::_1cEnterprise1cd => &_1C_ENTERPRISE_1CD,
        magika::ContentType::_1cEnterpriseCf => &_1C_ENTERPRISE_CF,
        magika::ContentType::_2dkgt2Demo => &_2DKGT2_DEMO,
        magika::ContentType::_3ds => &_3DS,
        magika::ContentType::_3dsm => &_3DSM,
        magika::ContentType::_3dsx => &_3DSX,
        magika::ContentType::_3gp => &_3GP,
        magika::ContentType::_3mf => &_3MF,
        magika::ContentType::A2r => &A2R,
        magika::ContentType::A78Rom => &A78_ROM,
        magika::ContentType::Aapt2Flat => &AAPT2_FLAT,
        magika::ContentType::AaruDicf => &AARU_DICF,
        magika::ContentType::Abcomp => &ABCOMP,
        magika::ContentType::AbletonAmxd => &ABLETON_AMXD,
        magika::ContentType::AbletonAsd => &ABLETON_ASD,
        magika::ContentType::AbsoluteDbQc => &ABSOLUTE_DB_QC,
        magika::ContentType::Ac1dPacker => &AC1D_PACKER,
        magika::ContentType::Access => &ACCESS,
        magika::ContentType::Ace => &ACE,
        magika::ContentType::Acr5Icdb => &ACR5_ICDB,
        magika::ContentType::AcronisMsgCatalog => &ACRONIS_MSG_CATALOG,
        magika::ContentType::AcronisTib => &ACRONIS_TIB,
        magika::ContentType::ActaOutline => &ACTA_OUTLINE,
        magika::ContentType::ActionamicsAst => &ACTIONAMICS_AST,
        magika::ContentType::AdexplorerDat => &ADEXPLORER_DAT,
        magika::ContentType::AdlibA2m => &ADLIB_A2M,
        magika::ContentType::AdlibBam => &ADLIB_BAM,
        magika::ContentType::AdlibCmf => &ADLIB_CMF,
        magika::ContentType::AdlibDro => &ADLIB_DRO,
        magika::ContentType::AdlibFernandez => &ADLIB_FERNANDEZ,
        magika::ContentType::AdlibRad => &ADLIB_RAD,
        magika::ContentType::AdlibRaw => &ADLIB_RAW,
        magika::ContentType::AdlibRol => &ADLIB_ROL,
        magika::ContentType::AdlibSbi => &ADLIB_SBI,
        magika::ContentType::AdobeAsnSif => &ADOBE_ASN_SIF,
        magika::ContentType::AdobeRna => &ADOBE_RNA,
        magika::ContentType::AdobeSwatchExchangeAse => &ADOBE_SWATCH_EXCHANGE_ASE,
        magika::ContentType::AdobeTechpoolTutorialTpp => &ADOBE_TECHPOOL_TUTORIAL_TPP,
        magika::ContentType::AdriftTaf => &ADRIFT_TAF,
        magika::ContentType::AdvancedForensicFormatAff => &ADVANCED_FORENSIC_FORMAT_AFF,
        magika::ContentType::Advsys => &ADVSYS,
        magika::ContentType::AeroflyTmt => &AEROFLY_TMT,
        magika::ContentType::Aff => &AFF,
        magika::ContentType::AgsTra => &AGS_TRA,
        magika::ContentType::AgtAgx => &AGT_AGX,
        magika::ContentType::Ai => &AI,
        magika::ContentType::AidaimSfs => &AIDAIM_SFS,
        magika::ContentType::Aidl => &AIDL,
        magika::ContentType::Alan3Game => &ALAN3_GAME,
        magika::ContentType::AlchGal => &ALCH_GAL,
        magika::ContentType::Alembic => &ALEMBIC,
        magika::ContentType::AllSoundTracker => &ALL_SOUND_TRACKER,
        magika::ContentType::AllegroFasl => &ALLEGRO_FASL,
        magika::ContentType::AllwinnerEgon => &ALLWINNER_EGON,
        magika::ContentType::AllwinnerImagewty => &ALLWINNER_IMAGEWTY,
        magika::ContentType::AlphaBmp => &ALPHA_BMP,
        magika::ContentType::AlphaWrgg => &ALPHA_WRGG,
        magika::ContentType::AlsaSocTopology => &ALSA_SOC_TOPOLOGY,
        magika::ContentType::AltoAl => &ALTO_AL,
        magika::ContentType::AltoDsk => &ALTO_DSK,
        magika::ContentType::AltoRun => &ALTO_RUN,
        magika::ContentType::AltoWidths => &ALTO_WIDTHS,
        magika::ContentType::Alz => &ALZ,
        magika::ContentType::Am6Amp => &AM6_AMP,
        magika::ContentType::AmAcetracker => &AM_ACETRACKER,
        magika::ContentType::AmComposer => &AM_COMPOSER,
        magika::ContentType::AmdMicrocode => &AMD_MICROCODE,
        magika::ContentType::AmdPspFirmware => &AMD_PSP_FIRMWARE,
        magika::ContentType::AmdgpuUcode => &AMDGPU_UCODE,
        magika::ContentType::AmfDsmi => &AMF_DSMI,
        magika::ContentType::AmgArchive => &AMG_ARCHIVE,
        magika::ContentType::AmigaAhx => &AMIGA_AHX,
        magika::ContentType::AmigaCdtvResidentTm => &AMIGA_CDTV_RESIDENT_TM,
        magika::ContentType::AmigaDbm => &AMIGA_DBM,
        magika::ContentType::AmigaFc => &AMIGA_FC,
        magika::ContentType::AmigaOkt => &AMIGA_OKT,
        magika::ContentType::AmigaPackdev => &AMIGA_PACKDEV,
        magika::ContentType::AmigaSquash => &AMIGA_SQUASH,
        magika::ContentType::AmigaZoom => &AMIGA_ZOOM,
        magika::ContentType::AmlogicMpt => &AMLOGIC_MPT,
        magika::ContentType::AmlogicUpgrade => &AMLOGIC_UPGRADE,
        magika::ContentType::AmplesoundUmsk => &AMPLESOUND_UMSK,
        magika::ContentType::AmsExtreme => &AMS_EXTREME,
        magika::ContentType::AmxSynth => &AMX_SYNTH,
        magika::ContentType::AncCruncher => &ANC_CRUNCHER,
        magika::ContentType::AndersOland => &ANDERS_OLAND,
        magika::ContentType::AndroidArtProfile => &ANDROID_ART_PROFILE,
        magika::ContentType::AndroidBootloaderImg => &ANDROID_BOOTLOADER_IMG,
        magika::ContentType::AndroidDtbo => &ANDROID_DTBO,
        magika::ContentType::AndroidHyb => &ANDROID_HYB,
        magika::ContentType::AndroidOta => &ANDROID_OTA,
        magika::ContentType::AndroidSparse => &ANDROID_SPARSE,
        magika::ContentType::AndroidVbmeta => &ANDROID_VBMETA,
        magika::ContentType::AndroidWinscopeTrace => &ANDROID_WINSCOPE_TRACE,
        magika::ContentType::Ani => &ANI,
        magika::ContentType::AnsiCsiArt => &ANSI_CSI_ART,
        magika::ContentType::AnsiTape => &ANSI_TAPE,
        magika::ContentType::AoeDrs => &AOE_DRS,
        magika::ContentType::AolArt => &AOL_ART,
        magika::ContentType::AonFm => &AON_FM,
        magika::ContentType::AorDv10Fw => &AOR_DV10_FW,
        magika::ContentType::Aout => &AOUT,
        magika::ContentType::ApisIqIce => &APIS_IQ_ICE,
        magika::ContentType::Apk => &APK,
        magika::ContentType::ApolloSysboot => &APOLLO_SYSBOOT,
        magika::ContentType::Apple8900 => &APPLE_8900,
        magika::ContentType::AppleAea => &APPLE_AEA,
        magika::ContentType::AppleArchive => &APPLE_ARCHIVE,
        magika::ContentType::AppleAsl => &APPLE_ASL,
        magika::ContentType::AppleBinaryIi => &APPLE_BINARY_II,
        magika::ContentType::AppleFairplaySupf => &APPLE_FAIRPLAY_SUPF,
        magika::ContentType::AppleFairplaySupp => &APPLE_FAIRPLAY_SUPP,
        magika::ContentType::AppleMkext => &APPLE_MKEXT,
        magika::ContentType::AppleNibarchive => &APPLE_NIBARCHIVE,
        magika::ContentType::AppleSecuritydBlob => &APPLE_SECURITYD_BLOB,
        magika::ContentType::AppleSparseimage => &APPLE_SPARSEIMAGE,
        magika::ContentType::AppleSpotlightStore => &APPLE_SPOTLIGHT_STORE,
        magika::ContentType::AppleVmp4 => &APPLE_VMP4,
        magika::ContentType::Applebplist => &APPLEBPLIST,
        magika::ContentType::Appledouble => &APPLEDOUBLE,
        magika::ContentType::Appleplist => &APPLEPLIST,
        magika::ContentType::Applesingle => &APPLESINGLE,
        magika::ContentType::AprosysAps => &APROSYS_APS,
        magika::ContentType::ApsN64Patch => &APS_N64_PATCH,
        magika::ContentType::Ar => &AR,
        magika::ContentType::Arc => &ARC,
        magika::ContentType::ArcCastpart => &ARC_CASTPART,
        magika::ContentType::ArcadyanFw => &ARCADYAN_FW,
        magika::ContentType::Arcgrid => &ARCGRID,
        magika::ContentType::ArchiveHashRtm => &ARCHIVE_HASH_RTM,
        magika::ContentType::ArcsoftUiAui => &ARCSOFT_UI_AUI,
        magika::ContentType::Arcv => &ARCV,
        magika::ContentType::ArgonautAsf => &ARGONAUT_ASF,
        magika::ContentType::ArgonautBrp => &ARGONAUT_BRP,
        magika::ContentType::Arj => &ARJ,
        magika::ContentType::ArmMaliCsffw => &ARM_MALI_CSFFW,
        magika::ContentType::ArmVectorRom => &ARM_VECTOR_ROM,
        magika::ContentType::Arrow => &ARROW,
        magika::ContentType::Art => &ART,
        magika::ContentType::ArtOfNoiseAon => &ART_OF_NOISE_AON,
        magika::ContentType::Asd => &ASD,
        magika::ContentType::Asdf => &ASDF,
        magika::ContentType::Asf => &ASF,
        magika::ContentType::AshampooAshlang => &ASHAMPOO_ASHLANG,
        magika::ContentType::Asksam => &ASKSAM,
        magika::ContentType::Asm => &ASM,
        magika::ContentType::Asp => &ASP,
        magika::ContentType::AssettoCorsaKn5 => &ASSETTO_CORSA_KN5,
        magika::ContentType::AssettocorsaAcd => &ASSETTOCORSA_ACD,
        magika::ContentType::Astc => &ASTC,
        magika::ContentType::AstmE57 => &ASTM_E57,
        magika::ContentType::AsylumMusicFormatAmf => &ASYLUM_MUSIC_FORMAT_AMF,
        magika::ContentType::AszPackage => &ASZ_PACKAGE,
        magika::ContentType::AtariCad3d2 => &ATARI_CAD3D2,
        magika::ContentType::AtariCrackart => &ATARI_CRACKART,
        magika::ContentType::AtariGfaBasic => &ATARI_GFA_BASIC,
        magika::ContentType::AtariMpp => &ATARI_MPP,
        magika::ContentType::AtariMsa => &ATARI_MSA,
        magika::ContentType::AtariSap => &ATARI_SAP,
        magika::ContentType::AtariStGuideHyp => &ATARI_ST_GUIDE_HYP,
        magika::ContentType::AtariStx => &ATARI_STX,
        magika::ContentType::AtariYm => &ATARI_YM,
        magika::ContentType::AtlusGfs => &ATLUS_GFS,
        magika::ContentType::Atomik => &ATOMIK,
        magika::ContentType::Au => &AU,
        magika::ContentType::Audfprint => &AUDFPRINT,
        magika::ContentType::AudibleAa => &AUDIBLE_AA,
        magika::ContentType::AudioSculptureAs => &AUDIO_SCULPTURE_AS,
        magika::ContentType::AulDsc => &AUL_DSC,
        magika::ContentType::AulTimesync => &AUL_TIMESYNC,
        magika::ContentType::AulUuidtext => &AUL_UUIDTEXT,
        magika::ContentType::AuroradspState => &AURORADSP_STATE,
        magika::ContentType::AutelEcc => &AUTEL_ECC,
        magika::ContentType::AutocadVlx => &AUTOCAD_VLX,
        magika::ContentType::Autohotkey => &AUTOHOTKEY,
        magika::ContentType::Autoit => &AUTOIT,
        magika::ContentType::AutolispFas => &AUTOLISP_FAS,
        magika::ContentType::AutorunMaestroMenuFileMnu => &AUTORUN_MAESTRO_MENU_FILE_MNU,
        magika::ContentType::AvastVpu => &AVAST_VPU,
        magika::ContentType::AvchdThumbnailIndexFileTid => &AVCHD_THUMBNAIL_INDEX_FILE_TID,
        magika::ContentType::Avi => &AVI,
        magika::ContentType::Avif => &AVIF,
        magika::ContentType::AvrAudio => &AVR_AUDIO,
        magika::ContentType::Avro => &AVRO,
        magika::ContentType::AwardBiosStf => &AWARD_BIOS_STF,
        magika::ContentType::Awk => &AWK,
        magika::ContentType::BahnNt3 => &BAHN_NT3,
        magika::ContentType::Bai => &BAI,
        magika::ContentType::Bam => &BAM,
        magika::ContentType::BandaiNamcoNus3 => &BANDAI_NAMCO_NUS3,
        magika::ContentType::BannermaniaFnt => &BANNERMANIA_FNT,
        magika::ContentType::BartenderBtw => &BARTENDER_BTW,
        magika::ContentType::Basis => &BASIS,
        magika::ContentType::Batch => &BATCH,
        magika::ContentType::Bazel => &BAZEL,
        magika::ContentType::Bbsong => &BBSONG,
        magika::ContentType::BclBfc => &BCL_BFC,
        magika::ContentType::Bcres => &BCRES,
        magika::ContentType::BeagleworksDoc => &BEAGLEWORKS_DOC,
        magika::ContentType::Beam => &BEAM,
        magika::ContentType::BeaverSweeper => &BEAVER_SWEEPER,
        magika::ContentType::BerkeleyVfont => &BERKELEY_VFONT,
        magika::ContentType::Berkeleydb => &BERKELEYDB,
        magika::ContentType::BerotrackerBrt => &BEROTRACKER_BRT,
        magika::ContentType::BethesdaBgsm => &BETHESDA_BGSM,
        magika::ContentType::BethesdaEsp => &BETHESDA_ESP,
        magika::ContentType::BethesdaFuz => &BETHESDA_FUZ,
        magika::ContentType::BethesdaPex => &BETHESDA_PEX,
        magika::ContentType::BethesdaVid => &BETHESDA_VID,
        magika::ContentType::BeyondtrustBrdfRdf => &BEYONDTRUST_BRDF_RDF,
        magika::ContentType::BeyondtrustSccUi => &BEYONDTRUST_SCC_UI,
        magika::ContentType::BfiVideo => &BFI_VIDEO,
        magika::ContentType::Bfres => &BFRES,
        magika::ContentType::BgiChr => &BGI_CHR,
        magika::ContentType::Bh => &BH,
        magika::ContentType::Bib => &BIB,
        magika::ContentType::Bif => &BIF,
        magika::ContentType::BigworldPackedXml => &BIGWORLD_PACKED_XML,
        magika::ContentType::BimMetadataFile => &BIM_METADATA_FILE,
        magika::ContentType::Binexport2 => &BINEXPORT2,
        magika::ContentType::Bink => &BINK,
        magika::ContentType::Biosemi24BitBdf => &BIOSEMI_24_BIT_BDF,
        magika::ContentType::BiowareFxm => &BIOWARE_FXM,
        magika::ContentType::BiowareGff => &BIOWARE_GFF,
        magika::ContentType::BiowareKey => &BIOWARE_KEY,
        magika::ContentType::BiowareNcs => &BIOWARE_NCS,
        magika::ContentType::BiowareRim => &BIOWARE_RIM,
        magika::ContentType::BiowareTlk => &BIOWARE_TLK,
        magika::ContentType::BitdefenderAvxs => &BITDEFENDER_AVXS,
        magika::ContentType::BittorrentBtdb => &BITTORRENT_BTDB,
        magika::ContentType::BitwigPreset => &BITWIG_PRESET,
        magika::ContentType::BiztreeBtd => &BIZTREE_BTD,
        magika::ContentType::BlackdragonZl => &BLACKDRAGON_ZL,
        magika::ContentType::Blend => &BLEND,
        magika::ContentType::Blitz3dFileFormatB3d => &BLITZ3D_FILE_FORMAT_B3D,
        magika::ContentType::BlizzardM3 => &BLIZZARD_M3,
        magika::ContentType::BlizzardWooTbl => &BLIZZARD_WOO_TBL,
        magika::ContentType::BlizzardWowBls => &BLIZZARD_WOW_BLS,
        magika::ContentType::Blosc => &BLOSC,
        magika::ContentType::BluebyteTpwm => &BLUEBYTE_TPWM,
        magika::ContentType::BluestripeHb => &BLUESTRIPE_HB,
        magika::ContentType::BlurayClpi => &BLURAY_CLPI,
        magika::ContentType::BlurayMobj => &BLURAY_MOBJ,
        magika::ContentType::BlurayMpls => &BLURAY_MPLS,
        magika::ContentType::BmfFont => &BMF_FONT,
        magika::ContentType::Bmp => &BMP,
        magika::ContentType::Bmx => &BMX,
        magika::ContentType::Bng => &BNG,
        magika::ContentType::BnsDialogCbf => &BNS_DIALOG_CBF,
        magika::ContentType::Bntl => &BNTL,
        magika::ContentType::BochsDisk => &BOCHS_DISK,
        magika::ContentType::BodyslideOsd => &BODYSLIDE_OSD,
        magika::ContentType::BohemiaP3d => &BOHEMIA_P3D,
        magika::ContentType::BohemiaPaa => &BOHEMIA_PAA,
        magika::ContentType::BohemiaRtm => &BOHEMIA_RTM,
        magika::ContentType::BoltDb => &BOLT_DB,
        magika::ContentType::Bom => &BOM,
        magika::ContentType::BoomtrackerCff => &BOOMTRACKER_CFF,
        magika::ContentType::BorlandBdeLocaleBtl => &BORLAND_BDE_LOCALE_BTL,
        magika::ContentType::BorlandBgi => &BORLAND_BGI,
        magika::ContentType::BorlandDpmiOverlayOvy => &BORLAND_DPMI_OVERLAY_OVY,
        magika::ContentType::BorlandTpu => &BORLAND_TPU,
        magika::ContentType::BorlandTurboDebugHelpTdh => &BORLAND_TURBO_DEBUG_HELP_TDH,
        magika::ContentType::BorlandTurboHelpTch => &BORLAND_TURBO_HELP_TCH,
        magika::ContentType::Box8686f => &BOX86_86F,
        magika::ContentType::BoyscoutBsf => &BOYSCOUT_BSF,
        magika::ContentType::Bpg => &BPG,
        magika::ContentType::Bpl => &BPL,
        magika::ContentType::BrAs3trace => &BR_AS3TRACE,
        magika::ContentType::BroadcomCfe => &BROADCOM_CFE,
        magika::ContentType::BroadcomHcd => &BROADCOM_HCD,
        magika::ContentType::BroadcomStbBootloader => &BROADCOM_STB_BOOTLOADER,
        magika::ContentType::BroadcomTrx => &BROADCOM_TRX,
        magika::ContentType::BrotherPes => &BROTHER_PES,
        magika::ContentType::Brsar => &BRSAR,
        magika::ContentType::Brstm => &BRSTM,
        magika::ContentType::Brunsli => &BRUNSLI,
        magika::ContentType::Brwav => &BRWAV,
        magika::ContentType::BsaBa2 => &BSA_BA2,
        magika::ContentType::BsdDump => &BSD_DUMP,
        magika::ContentType::Bsdiff => &BSDIFF,
        magika::ContentType::Bson => &BSON,
        magika::ContentType::Bsp => &BSP,
        magika::ContentType::BtrfsSendStream => &BTRFS_SEND_STREAM,
        magika::ContentType::BunPile => &BUN_PILE,
        magika::ContentType::BurnpipeBa => &BURNPIPE_BA,
        magika::ContentType::Buzzic2 => &BUZZIC2,
        magika::ContentType::Bwd1Doc => &BWD1_DOC,
        magika::ContentType::Bzip => &BZIP,
        magika::ContentType::Bzip3 => &BZIP3,
        magika::ContentType::C => &C,
        magika::ContentType::C1581D81 => &C1581_D81,
        magika::ContentType::C3Maxfile => &C3_MAXFILE,
        magika::ContentType::C64Godot => &C64_GODOT,
        magika::ContentType::C64Tap => &C64_TAP,
        magika::ContentType::Cab => &CAB,
        magika::ContentType::CadcorpSisNol => &CADCORP_SIS_NOL,
        magika::ContentType::Caf => &CAF,
        magika::ContentType::CaffeineCscheme => &CAFFEINE_CSCHEME,
        magika::ContentType::CakewalkWrk => &CAKEWALK_WRK,
        magika::ContentType::Cal3d => &CAL3D,
        magika::ContentType::CalsDcl => &CALS_DCL,
        magika::ContentType::CanonVrd => &CANON_VRD,
        magika::ContentType::CapcomMtGmd => &CAPCOM_MT_GMD,
        magika::ContentType::CapcomMtMrl => &CAPCOM_MT_MRL,
        magika::ContentType::CapcomRePak => &CAPCOM_RE_PAK,
        magika::ContentType::CapsIpf => &CAPS_IPF,
        magika::ContentType::CarvVrac => &CARV_VRAC,
        magika::ContentType::CasioContainer => &CASIO_CONTAINER,
        magika::ContentType::CaslogDlog => &CASLOG_DLOG,
        magika::ContentType::Castledb => &CASTLEDB,
        magika::ContentType::Cat => &CAT,
        magika::ContentType::Catboost => &CATBOOST,
        magika::ContentType::CatiaDrawing => &CATIA_DRAWING,
        magika::ContentType::CausticPcmsynthWk => &CAUSTIC_PCMSYNTH_WK,
        magika::ContentType::Cazip => &CAZIP,
        magika::ContentType::CbaModule => &CBA_MODULE,
        magika::ContentType::Cbor => &CBOR,
        magika::ContentType::Ccz => &CCZ,
        magika::ContentType::CdesEncryptedEnc => &CDES_ENCRYPTED_ENC,
        magika::ContentType::Cdf => &CDF,
        magika::ContentType::Cdfm670 => &CDFM_670,
        magika::ContentType::CdtreeCd => &CDTREE_CD,
        magika::ContentType::Cdx => &CDX,
        magika::ContentType::CedictUnitU8 => &CEDICT_UNIT_U8,
        magika::ContentType::CeditUcviewHis => &CEDIT_UCVIEW_HIS,
        magika::ContentType::CeditUcviewHlp => &CEDIT_UCVIEW_HLP,
        magika::ContentType::CernRoot => &CERN_ROOT,
        magika::ContentType::CernZebraFz => &CERN_ZEBRA_FZ,
        magika::ContentType::CernZebraRz => &CERN_ZEBRA_RZ,
        magika::ContentType::CesiumB3dm => &CESIUM_B3DM,
        magika::ContentType::CesiumGeom => &CESIUM_GEOM,
        magika::ContentType::Cfl3 => &CFL3,
        magika::ContentType::ChapsTimetableTt => &CHAPS_TIMETABLE_TT,
        magika::ContentType::Chd => &CHD,
        magika::ContentType::ChemstationGasChromatographyMassMs => {
            &CHEMSTATION_GAS_CHROMATOGRAPHY_MASS_MS
        }
        magika::ContentType::ChessbaseTreeCtb => &CHESSBASE_TREE_CTB,
        magika::ContentType::ChiptrackerKris => &CHIPTRACKER_KRIS,
        magika::ContentType::Chm => &CHM,
        magika::ContentType::ChromeBdic => &CHROME_BDIC,
        magika::ContentType::ChromeCacheBlockfile => &CHROME_CACHE_BLOCKFILE,
        magika::ContentType::ChromeSimpleCache => &CHROME_SIMPLE_CACHE,
        magika::ContentType::ChromeSnss => &CHROME_SNSS,
        magika::ContentType::ChromeV4store => &CHROME_V4STORE,
        magika::ContentType::ChromeVlnk => &CHROME_VLNK,
        magika::ContentType::ChromeosFmap => &CHROMEOS_FMAP,
        magika::ContentType::ChromeosVb21 => &CHROMEOS_VB21,
        magika::ContentType::ChromeosVboot => &CHROMEOS_VBOOT,
        magika::ContentType::ChronomasterDfa => &CHRONOMASTER_DFA,
        magika::ContentType::ChxFont => &CHX_FONT,
        magika::ContentType::CigamesDpk4 => &CIGAMES_DPK4,
        magika::ContentType::Cinema4d => &CINEMA4D,
        magika::ContentType::Cineon => &CINEON,
        magika::ContentType::Civ3Bic => &CIV3_BIC,
        magika::ContentType::ClangOffloadBundle => &CLANG_OFFLOAD_BUNDLE,
        magika::ContentType::ClarionHlp => &CLARION_HLP,
        magika::ContentType::ClarionMem => &CLARION_MEM,
        magika::ContentType::ClarisResolve => &CLARIS_RESOLVE,
        magika::ContentType::ClickteamCcn => &CLICKTEAM_CCN,
        magika::ContentType::ClickteamMfa => &CLICKTEAM_MFA,
        magika::ContentType::ClipStudioPaint => &CLIP_STUDIO_PAINT,
        magika::ContentType::ClipperNtx => &CLIPPER_NTX,
        magika::ContentType::Clojure => &CLOJURE,
        magika::ContentType::Cmake => &CMAKE,
        magika::ContentType::CmapCmp => &CMAP_CMP,
        magika::ContentType::Cmx => &CMX,
        magika::ContentType::Cobol => &COBOL,
        magika::ContentType::CodFastfile => &COD_FASTFILE,
        magika::ContentType::CodGscc => &COD_GSCC,
        magika::ContentType::Coff => &COFF,
        magika::ContentType::Coffeescript => &COFFEESCRIPT,
        magika::ContentType::CoktelStk2 => &COKTEL_STK2,
        magika::ContentType::Collada => &COLLADA,
        magika::ContentType::CombiloaderBcf => &COMBILOADER_BCF,
        magika::ContentType::CommonInstrumentCi1 => &COMMON_INSTRUMENT_CI1,
        magika::ContentType::CommonLoudspeakerCf1 => &COMMON_LOUDSPEAKER_CF1,
        magika::ContentType::Composer667 => &COMPOSER_667,
        magika::ContentType::ConsDictNx2 => &CONS_DICT_NX2,
        magika::ContentType::ConsultantplusDt5 => &CONSULTANTPLUS_DT5,
        magika::ContentType::CorebootCbfs => &COREBOOT_CBFS,
        magika::ContentType::CosmigoBrc => &COSMIGO_BRC,
        magika::ContentType::CosmigoStm => &COSMIGO_STM,
        magika::ContentType::Cpio => &CPIO,
        magika::ContentType::Cpk => &CPK,
        magika::ContentType::Cpp => &CPP,
        magika::ContentType::Cpx => &CPX,
        magika::ContentType::Cram => &CRAM,
        magika::ContentType::Cramfs => &CRAMFS,
        magika::ContentType::CrashpadCpad => &CRASHPAD_CPAD,
        magika::ContentType::CreativeXfiFw => &CREATIVE_XFI_FW,
        magika::ContentType::CriHca => &CRI_HCA,
        magika::ContentType::CriUsm => &CRI_USM,
        magika::ContentType::CriwareAwb => &CRIWARE_AWB,
        magika::ContentType::Crt => &CRT,
        magika::ContentType::CrunchMania => &CRUNCH_MANIA,
        magika::ContentType::Crush => &CRUSH,
        magika::ContentType::Crx => &CRX,
        magika::ContentType::CryengineDdsMip6 => &CRYENGINE_DDS_MIP6,
        magika::ContentType::CryengineSoc => &CRYENGINE_SOC,
        magika::ContentType::CryoApc => &CRYO_APC,
        magika::ContentType::CryoBigfile => &CRYO_BIGFILE,
        magika::ContentType::CryoClibVox => &CRYO_CLIB_VOX,
        magika::ContentType::CryoHnm4 => &CRYO_HNM4,
        magika::ContentType::CryptoapiKeyblob => &CRYPTOAPI_KEYBLOB,
        magika::ContentType::CrystalReportsV4Rpt => &CRYSTAL_REPORTS_V4_RPT,
        magika::ContentType::Cs => &CS,
        magika::ContentType::CsmanDat => &CSMAN_DAT,
        magika::ContentType::Cso => &CSO,
        magika::ContentType::Csproj => &CSPROJ,
        magika::ContentType::CsrDfu2 => &CSR_DFU2,
        magika::ContentType::Css => &CSS,
        magika::ContentType::Csv => &CSV,
        magika::ContentType::Ctm2Map => &CTM2_MAP,
        magika::ContentType::CtosRun => &CTOS_RUN,
        magika::ContentType::CtosVol => &CTOS_VOL,
        magika::ContentType::CtraderAlgo => &CTRADER_ALGO,
        magika::ContentType::CubicMxm => &CUBIC_MXM,
        magika::ContentType::Cubin => &CUBIN,
        magika::ContentType::CudaFatbin => &CUDA_FATBIN,
        magika::ContentType::CupsIpp => &CUPS_IPP,
        magika::ContentType::CxlHelp => &CXL_HELP,
        magika::ContentType::CyanMohawk => &CYAN_MOHAWK,
        magika::ContentType::CyberlinkKc => &CYBERLINK_KC,
        magika::ContentType::CyberpaintSeq => &CYBERPAINT_SEQ,
        magika::ContentType::CybertrackerC64 => &CYBERTRACKER_C64,
        magika::ContentType::CypressClmBlob => &CYPRESS_CLM_BLOB,
        magika::ContentType::CzHelp => &CZ_HELP,
        magika::ContentType::D3dIndexBufferIb => &D3D_INDEX_BUFFER_IB,
        magika::ContentType::DagorDxp => &DAGOR_DXP,
        magika::ContentType::DagorGrp => &DAGOR_GRP,
        magika::ContentType::DagorVromfs => &DAGOR_VROMFS,
        magika::ContentType::DaisydotNlq => &DAISYDOT_NLQ,
        magika::ContentType::Dan1Archive => &DAN1_ARCHIVE,
        magika::ContentType::DarkwaveDwp => &DARKWAVE_DWP,
        magika::ContentType::DarpaCgc => &DARPA_CGC,
        magika::ContentType::Dart => &DART,
        magika::ContentType::DartDsk => &DART_DSK,
        magika::ContentType::DatabentoDbn => &DATABENTO_DBN,
        magika::ContentType::DaveLoweDln => &DAVE_LOWE_DLN,
        magika::ContentType::DavidHanneyDh => &DAVID_HANNEY_DH,
        magika::ContentType::DavidWhittaker => &DAVID_WHITTAKER,
        magika::ContentType::DawesomeAbyss2 => &DAWESOME_ABYSS2,
        magika::ContentType::DawesomeKult => &DAWESOME_KULT,
        magika::ContentType::Db2 => &DB2,
        magika::ContentType::Dbase => &DBASE,
        magika::ContentType::DbaseLbl => &DBASE_LBL,
        magika::ContentType::Dbg => &DBG,
        magika::ContentType::Dbpf => &DBPF,
        magika::ContentType::Dca => &DCA,
        magika::ContentType::DcsWorldEdm => &DCS_WORLD_EDM,
        magika::ContentType::Ddi1Bgra => &DDI1_BGRA,
        magika::ContentType::Deb => &DEB,
        magika::ContentType::DecRt11 => &DEC_RT11,
        magika::ContentType::DecXxdp => &DEC_XXDP,
        magika::ContentType::DefiniconDsi32E32 => &DEFINICON_DSI32_E32,
        magika::ContentType::DefyDt => &DEFY_DT,
        magika::ContentType::Degas => &DEGAS,
        magika::ContentType::DellDiagnosticsMlm => &DELL_DIAGNOSTICS_MLM,
        magika::ContentType::DelphiCardPublisherLcp => &DELPHI_CARD_PUBLISHER_LCP,
        magika::ContentType::DelphiCds => &DELPHI_CDS,
        magika::ContentType::DelphiDcu => &DELPHI_DCU,
        magika::ContentType::DeltaRpm => &DELTA_RPM,
        magika::ContentType::DemomakerFlm => &DEMOMAKER_FLM,
        magika::ContentType::DescentHog => &DESCENT_HOG,
        magika::ContentType::DesiDdf => &DESI_DDF,
        magika::ContentType::DesktopTracker => &DESKTOP_TRACKER,
        magika::ContentType::DesqviewHlp => &DESQVIEW_HLP,
        magika::ContentType::DesyFpackBos => &DESY_FPACK_BOS,
        magika::ContentType::Dex => &DEX,
        magika::ContentType::DgNovaRb => &DG_NOVA_RB,
        magika::ContentType::Diablo2D2s => &DIABLO2_D2S,
        magika::ContentType::Dicom => &DICOM,
        magika::ContentType::DieselGai => &DIESEL_GAI,
        magika::ContentType::Diff => &DIFF,
        magika::ContentType::DiffracplusRawData => &DIFFRACPLUS_RAW_DATA,
        magika::ContentType::DifstreamSvf => &DIFSTREAM_SVF,
        magika::ContentType::DigitalFm => &DIGITAL_FM,
        magika::ContentType::DigitalSonixDsc => &DIGITAL_SONIX_DSC,
        magika::ContentType::DimpsEmb => &DIMPS_EMB,
        magika::ContentType::DirectwaveDwp => &DIRECTWAVE_DWP,
        magika::ContentType::DirkBialluchSmp => &DIRK_BIALLUCH_SMP,
        magika::ContentType::DirkBialluchTpu => &DIRK_BIALLUCH_TPU,
        magika::ContentType::DiskdupeDdi => &DISKDUPE_DDI,
        magika::ContentType::DiskoveryScan => &DISKOVERY_SCAN,
        magika::ContentType::DlinkAih0 => &DLINK_AIH0,
        magika::ContentType::DlinkDeafbead => &DLINK_DEAFBEAD,
        magika::ContentType::DlinkFpkg => &DLINK_FPKG,
        magika::ContentType::DlinkMh01 => &DLINK_MH01,
        magika::ContentType::DlinkShrs => &DLINK_SHRS,
        magika::ContentType::DlinkTlv => &DLINK_TLV,
        magika::ContentType::Dm => &DM,
        magika::ContentType::DmfXtracker => &DMF_XTRACKER,
        magika::ContentType::Dmg => &DMG,
        magika::ContentType::Dms => &DMS,
        magika::ContentType::DmtfPldmFirmware => &DMTF_PLDM_FIRMWARE,
        magika::ContentType::Dmu => &DMU,
        magika::ContentType::DnaSequenceChromatogramFileScf => &DNA_SEQUENCE_CHROMATOGRAM_FILE_SCF,
        magika::ContentType::Doc => &DOC,
        magika::ContentType::Dockerfile => &DOCKERFILE,
        magika::ContentType::Docx => &DOCX,
        magika::ContentType::Dos16mBw => &DOS16M_BW,
        magika::ContentType::DosDim => &DOS_DIM,
        magika::ContentType::DosHayesMdm => &DOS_HAYES_MDM,
        magika::ContentType::Dosmbr => &DOSMBR,
        magika::ContentType::DotnetJitProfile => &DOTNET_JIT_PROFILE,
        magika::ContentType::Dotx => &DOTX,
        magika::ContentType::Dpx => &DPX,
        magika::ContentType::DragonbonesDbbin => &DRAGONBONES_DBBIN,
        magika::ContentType::DrumtrakerDtl => &DRUMTRAKER_DTL,
        magika::ContentType::DrumwareK1Bank => &DRUMWARE_K1_BANK,
        magika::ContentType::Dsdiff => &DSDIFF,
        magika::ContentType::DsfAudio => &DSF_AUDIO,
        magika::ContentType::DsgAtonce => &DSG_ATONCE,
        magika::ContentType::DskAct => &DSK_ACT,
        magika::ContentType::DsmDsik => &DSM_DSIK,
        magika::ContentType::DssDreamstation => &DSS_DREAMSTATION,
        magika::ContentType::Dsstore => &DSSTORE,
        magika::ContentType::Dsym => &DSYM,
        magika::ContentType::Dtb => &DTB,
        magika::ContentType::DtchChart => &DTCH_CHART,
        magika::ContentType::Dtm => &DTM,
        magika::ContentType::Duckdb => &DUCKDB,
        magika::ContentType::DuragerUvReal => &DURAGER_UV_REAL,
        magika::ContentType::DvVideo => &DV_VIDEO,
        magika::ContentType::DvdIfo => &DVD_IFO,
        magika::ContentType::Dwg => &DWG,
        magika::ContentType::DwinDgus3Icl => &DWIN_DGUS3_ICL,
        magika::ContentType::Dxa => &DXA,
        magika::ContentType::Dxf => &DXF,
        magika::ContentType::DxtkSpritefont => &DXTK_SPRITEFONT,
        magika::ContentType::DynamicStudioDsm => &DYNAMIC_STUDIO_DSM,
        magika::ContentType::EaAbk => &EA_ABK,
        magika::ContentType::EaBnk => &EA_BNK,
        magika::ContentType::EaEbo => &EA_EBO,
        magika::ContentType::EaFfn => &EA_FFN,
        magika::ContentType::EaFsh => &EA_FSH,
        magika::ContentType::EaSchl => &EA_SCHL,
        magika::ContentType::EaViv4Bh => &EA_VIV4_BH,
        magika::ContentType::EaracheEa => &EARACHE_EA,
        magika::ContentType::EaseGll => &EASE_GLL,
        magika::ContentType::EastwestTchkKey => &EASTWEST_TCHK_KEY,
        magika::ContentType::Ecm => &ECM,
        magika::ContentType::Ecw => &ECW,
        magika::ContentType::EdfMesh => &EDF_MESH,
        magika::ContentType::Edilzss => &EDILZSS,
        magika::ContentType::EdlibD00 => &EDLIB_D00,
        magika::ContentType::EdlibEdl => &EDLIB_EDL,
        magika::ContentType::Egg => &EGG,
        magika::ContentType::Elc => &ELC,
        magika::ContentType::Elf => &ELF,
        magika::ContentType::Elixir => &ELIXIR,
        magika::ContentType::Emd => &EMD,
        magika::ContentType::Emf => &EMF,
        magika::ContentType::Eml => &EML,
        magika::ContentType::Empty => &EMPTY,
        magika::ContentType::EmsV1 => &EMS_V1,
        magika::ContentType::EmsV6 => &EMS_V6,
        magika::ContentType::EmtexDotDriver => &EMTEX_DOT_DRIVER,
        magika::ContentType::EmtexFontLibrary => &EMTEX_FONT_LIBRARY,
        magika::ContentType::EmtexTcpCodepage => &EMTEX_TCP_CODEPAGE,
        magika::ContentType::Enc => &ENC,
        magika::ContentType::EncaseE01 => &ENCASE_E01,
        magika::ContentType::EncoreEnc => &ENCORE_ENC,
        magika::ContentType::EndnoteEnf => &ENDNOTE_ENF,
        magika::ContentType::EnsoniqPaf => &ENSONIQ_PAF,
        magika::ContentType::EntisMio => &ENTIS_MIO,
        magika::ContentType::Envision => &ENVISION,
        magika::ContentType::EnvisionSvf => &ENVISION_SVF,
        magika::ContentType::EoWebengine => &EO_WEBENGINE,
        magika::ContentType::Epd => &EPD,
        magika::ContentType::EplanEoxDb => &EPLAN_EOX_DB,
        magika::ContentType::EplybndsModel => &EPLYBNDS_MODEL,
        magika::ContentType::EpsDos => &EPS_DOS,
        magika::ContentType::Epub => &EPUB,
        magika::ContentType::Erb => &ERB,
        magika::ContentType::Erdas => &ERDAS,
        magika::ContentType::Erlang => &ERLANG,
        magika::ContentType::Ese => &ESE,
        magika::ContentType::EspParttable => &ESP_PARTTABLE,
        magika::ContentType::EthereumEra1 => &ETHEREUM_ERA1,
        magika::ContentType::Etx => &ETX,
        magika::ContentType::EuphonyEup => &EUPHONY_EUP,
        magika::ContentType::EurekalogEl => &EUREKALOG_EL,
        magika::ContentType::Evt => &EVT,
        magika::ContentType::Evtx => &EVTX,
        magika::ContentType::Evy => &EVY,
        magika::ContentType::ExbnEbn => &EXBN_EBN,
        magika::ContentType::Exiv2Exv => &EXIV2_EXV,
        magika::ContentType::ExodusSeco => &EXODUS_SECO,
        magika::ContentType::ExoticXad => &EXOTIC_XAD,
        magika::ContentType::Exr => &EXR,
        magika::ContentType::Ext4 => &EXT4,
        magika::ContentType::Ezart => &EZART,
        magika::ContentType::EznecEz => &EZNEC_EZ,
        magika::ContentType::Fabylon => &FABYLON,
        magika::ContentType::FacegenTri => &FACEGEN_TRI,
        magika::ContentType::FairyguiFui => &FAIRYGUI_FUI,
        magika::ContentType::FalcomMi => &FALCOM_MI,
        magika::ContentType::FanucPmcPatch => &FANUC_PMC_PATCH,
        magika::ContentType::FarbrauschV2m => &FARBRAUSCH_V2M,
        magika::ContentType::Fasta => &FASTA,
        magika::ContentType::FastgraphFgi => &FASTGRAPH_FGI,
        magika::ContentType::FastgraphPpr => &FASTGRAPH_PPR,
        magika::ContentType::FastgraphSpr => &FASTGRAPH_SPR,
        magika::ContentType::Fastq => &FASTQ,
        magika::ContentType::FbaFs => &FBA_FS,
        magika::ContentType::FbchunksNgp => &FBCHUNKS_NGP,
        magika::ContentType::FbsTimetableFpl => &FBS_TIMETABLE_FPL,
        magika::ContentType::Fbx => &FBX,
        magika::ContentType::Fds => &FDS,
        magika::ContentType::FieldlogFl => &FIELDLOG_FL,
        magika::ContentType::Fif => &FIF,
        magika::ContentType::FifaFifamod => &FIFA_FIFAMOD,
        magika::ContentType::FighterMakerKgt => &FIGHTER_MAKER_KGT,
        magika::ContentType::FigmaFig => &FIGMA_FIG,
        magika::ContentType::FilamentFilamat => &FILAMENT_FILAMAT,
        magika::ContentType::Filemaker => &FILEMAKER,
        magika::ContentType::FineprintFp => &FINEPRINT_FP,
        magika::ContentType::FioVerifyState => &FIO_VERIFY_STATE,
        magika::ContentType::Firebird => &FIREBIRD,
        magika::ContentType::Fit => &FIT,
        magika::ContentType::Fits => &FITS,
        magika::ContentType::FlStudioFlp => &FL_STUDIO_FLP,
        magika::ContentType::Flac => &FLAC,
        magika::ContentType::FlashSol => &FLASH_SOL,
        magika::ContentType::Flatgeobuf => &FLATGEOBUF,
        magika::ContentType::FlatpakBundle => &FLATPAK_BUNDLE,
        magika::ContentType::FlightsimNameAir => &FLIGHTSIM_NAME_AIR,
        magika::ContentType::FlirFpf => &FLIR_FPF,
        magika::ContentType::FlmobileFlmrack => &FLMOBILE_FLMRACK,
        magika::ContentType::Flog => &FLOG,
        magika::ContentType::FluentInterpIp => &FLUENT_INTERP_IP,
        magika::ContentType::Flv => &FLV,
        magika::ContentType::FmFmf => &FM_FMF,
        magika::ContentType::FmKingtracker => &FM_KINGTRACKER,
        magika::ContentType::FmTracker => &FM_TRACKER,
        magika::ContentType::Fmc => &FMC,
        magika::ContentType::FmodFev => &FMOD_FEV,
        magika::ContentType::FmpM => &FMP_M,
        magika::ContentType::FmtownsEuphonyPmb => &FMTOWNS_EUPHONY_PMB,
        magika::ContentType::Fog => &FOG,
        magika::ContentType::FontconfigCache => &FONTCONFIG_CACHE,
        magika::ContentType::Fortran => &FORTRAN,
        magika::ContentType::FourdosHelp => &FOURDOS_HELP,
        magika::ContentType::FoxproFormSct => &FOXPRO_FORM_SCT,
        magika::ContentType::Fpx => &FPX,
        magika::ContentType::Freearc => &FREEARC,
        magika::ContentType::FreebasicHf => &FREEBASIC_HF,
        magika::ContentType::FreedomForceSav => &FREEDOM_FORCE_SAV,
        magika::ContentType::FromsoftBinder => &FROMSOFT_BINDER,
        magika::ContentType::FromsoftDcx => &FROMSOFT_DCX,
        magika::ContentType::FromsoftFlver => &FROMSOFT_FLVER,
        magika::ContentType::FromsoftTae => &FROMSOFT_TAE,
        magika::ContentType::FrostyFbmod => &FROSTY_FBMOD,
        magika::ContentType::FsecureFs02Mf => &FSECURE_FS02_MF,
        magika::ContentType::FtkAd1 => &FTK_AD1,
        magika::ContentType::Ftm => &FTM,
        magika::ContentType::Ftmn => &FTMN,
        magika::ContentType::FullwriteDoc => &FULLWRITE_DOC,
        magika::ContentType::FunktrackerFnk => &FUNKTRACKER_FNK,
        magika::ContentType::FutureComposerBsi => &FUTURE_COMPOSER_BSI,
        magika::ContentType::G9bBitmap => &G9B_BITMAP,
        magika::ContentType::GalaxyMusicMuse => &GALAXY_MUSIC_MUSE,
        magika::ContentType::GameMusicCreator => &GAME_MUSIC_CREATOR,
        magika::ContentType::GameboyGbr => &GAMEBOY_GBR,
        magika::ContentType::GameboyGbs => &GAMEBOY_GBS,
        magika::ContentType::GamemakerGmk => &GAMEMAKER_GMK,
        magika::ContentType::GarminGmn => &GARMIN_GMN,
        magika::ContentType::GbaGbfs => &GBA_GBFS,
        magika::ContentType::Gca => &GCA,
        magika::ContentType::Gdm => &GDM,
        magika::ContentType::GeaArchive => &GEA_ARCHIVE,
        magika::ContentType::GeckoStartupcache => &GECKO_STARTUPCACHE,
        magika::ContentType::GemVdi => &GEM_VDI,
        magika::ContentType::Gemfile => &GEMFILE,
        magika::ContentType::Gemspec => &GEMSPEC,
        magika::ContentType::GenesisGym => &GENESIS_GYM,
        magika::ContentType::Genh => &GENH,
        magika::ContentType::GeoGtx => &GEO_GTX,
        magika::ContentType::Geojson => &GEOJSON,
        magika::ContentType::Geopackage => &GEOPACKAGE,
        magika::ContentType::GeosoftGx => &GEOSOFT_GX,
        magika::ContentType::GeoworksGeos => &GEOWORKS_GEOS,
        magika::ContentType::GettextMo => &GETTEXT_MO,
        magika::ContentType::Ggml => &GGML,
        magika::ContentType::Gguf => &GGUF,
        magika::ContentType::GhcHi => &GHC_HI,
        magika::ContentType::GhidraBf => &GHIDRA_BF,
        magika::ContentType::GiantsGdm => &GIANTS_GDM,
        magika::ContentType::GiantsGrle => &GIANTS_GRLE,
        magika::ContentType::Gif => &GIF,
        magika::ContentType::GitIndex => &GIT_INDEX,
        magika::ContentType::GitPackIdx => &GIT_PACK_IDX,
        magika::ContentType::GitPackRev => &GIT_PACK_REV,
        magika::ContentType::Gitattributes => &GITATTRIBUTES,
        magika::ContentType::Gitmodules => &GITMODULES,
        magika::ContentType::GlibcLdSoCache => &GLIBC_LD_SO_CACHE,
        magika::ContentType::Glide3df => &GLIDE_3DF,
        magika::ContentType::Gltf => &GLTF,
        magika::ContentType::Gluemon => &GLUEMON,
        magika::ContentType::Glulx => &GLULX,
        magika::ContentType::Gma => &GMA,
        magika::ContentType::Gml => &GML,
        magika::ContentType::GnomeKeyring => &GNOME_KEYRING,
        magika::ContentType::GnuLocateDb => &GNU_LOCATE_DB,
        magika::ContentType::GnunetGnd => &GNUNET_GND,
        magika::ContentType::Go => &GO,
        magika::ContentType::GoTelemetryCount => &GO_TELEMETRY_COUNT,
        magika::ContentType::Goattracker => &GOATTRACKER,
        magika::ContentType::GobjectTypelib => &GOBJECT_TYPELIB,
        magika::ContentType::GodotGdec => &GODOT_GDEC,
        magika::ContentType::GodotPck => &GODOT_PCK,
        magika::ContentType::GodotRsrc => &GODOT_RSRC,
        magika::ContentType::Goff => &GOFF,
        magika::ContentType::GoldsrcWad3 => &GOLDSRC_WAD3,
        magika::ContentType::GpgTrustdb => &GPG_TRUSTDB,
        magika::ContentType::GpoRegistryPol => &GPO_REGISTRY_POL,
        magika::ContentType::Gpx => &GPX,
        magika::ContentType::Gradle => &GRADLE,
        magika::ContentType::Grafx2Pkm => &GRAFX2_PKM,
        magika::ContentType::GraoumfGt2 => &GRAOUMF_GT2,
        magika::ContentType::Grib => &GRIB,
        magika::ContentType::Groovy => &GROOVY,
        magika::ContentType::GsddGdd => &GSDD_GDD,
        magika::ContentType::GssiDzt => &GSSI_DZT,
        magika::ContentType::Gsym => &GSYM,
        magika::ContentType::GtGameSystemsDux => &GT_GAME_SYSTEMS_DUX,
        magika::ContentType::GtaIfp => &GTA_IFP,
        magika::ContentType::Guemap => &GUEMAP,
        magika::ContentType::GuitarProGpx => &GUITAR_PRO_GPX,
        magika::ContentType::GusPat => &GUS_PAT,
        magika::ContentType::Gx1 => &GX1,
        magika::ContentType::Gx2Bitmap => &GX2_BITMAP,
        magika::ContentType::Gzip => &GZIP,
        magika::ContentType::H2 => &H2,
        magika::ContentType::H5 => &H5,
        magika::ContentType::HaArchive => &HA_ARCHIVE,
        magika::ContentType::HaikuVectorIconFormatHvif => &HAIKU_VECTOR_ICON_FORMAT_HVIF,
        magika::ContentType::HaloBlf => &HALO_BLF,
        magika::ContentType::Halpst => &HALPST,
        magika::ContentType::Handlebars => &HANDLEBARS,
        magika::ContentType::HanmacWord => &HANMAC_WORD,
        magika::ContentType::Harvard => &HARVARD,
        magika::ContentType::Haskell => &HASKELL,
        magika::ContentType::HavokHkx => &HAVOK_HKX,
        magika::ContentType::Hcl => &HCL,
        magika::ContentType::Hdf4 => &HDF4,
        magika::ContentType::Hdiffpatch => &HDIFFPATCH,
        magika::ContentType::HdrRgbe => &HDR_RGBE,
        magika::ContentType::Heif => &HEIF,
        magika::ContentType::HeliosSave => &HELIOS_SAVE,
        magika::ContentType::HelppcIndex => &HELPPC_INDEX,
        magika::ContentType::HemeraHta => &HEMERA_HTA,
        magika::ContentType::HeradAdlib => &HERAD_ADLIB,
        magika::ContentType::HerculesDasd => &HERCULES_DASD,
        magika::ContentType::Hermes => &HERMES,
        magika::ContentType::Hes => &HES,
        magika::ContentType::Hfe => &HFE,
        magika::ContentType::Hfs => &HFS,
        magika::ContentType::HfstOlTransducer => &HFST_OL_TRANSDUCER,
        magika::ContentType::HhdLpack => &HHD_LPACK,
        magika::ContentType::HiewExt => &HIEW_EXT,
        magika::ContentType::HighlogicFcp => &HIGHLOGIC_FCP,
        magika::ContentType::HikvisionFirmware => &HIKVISION_FIRMWARE,
        magika::ContentType::Hipc => &HIPC,
        magika::ContentType::HippelHip => &HIPPEL_HIP,
        magika::ContentType::HisMastersNoise => &HIS_MASTERS_NOISE,
        magika::ContentType::HitachiHrf => &HITACHI_HRF,
        magika::ContentType::HivelyTracker => &HIVELY_TRACKER,
        magika::ContentType::Hlp => &HLP,
        magika::ContentType::HolyNoise => &HOLY_NOISE,
        magika::ContentType::HomeworldLif => &HOMEWORLD_LIF,
        magika::ContentType::HomeworldMex => &HOMEWORLD_MEX,
        magika::ContentType::HoopsHsf => &HOOPS_HSF,
        magika::ContentType::Hp2100Abs => &HP2100_ABS,
        magika::ContentType::Hp2100Rel => &HP2100_REL,
        magika::ContentType::Hp48Bin => &HP48_BIN,
        magika::ContentType::HpBdl => &HP_BDL,
        magika::ContentType::HpIpkg => &HP_IPKG,
        magika::ContentType::HpPalmtopExm => &HP_PALMTOP_EXM,
        magika::ContentType::HpPclSoftfont => &HP_PCL_SOFTFONT,
        magika::ContentType::Hpdt => &HPDT,
        magika::ContentType::HpuxSom => &HPUX_SOM,
        magika::ContentType::HscTracker => &HSC_TRACKER,
        magika::ContentType::HsiJpeg => &HSI_JPEG,
        magika::ContentType::HsiRaw => &HSI_RAW,
        magika::ContentType::Hta => &HTA,
        magika::ContentType::Htaccess => &HTACCESS,
        magika::ContentType::Html => &HTML,
        magika::ContentType::HuaweiUpdateApp => &HUAWEI_UPDATE_APP,
        magika::ContentType::Hve => &HVE,
        magika::ContentType::Hwp => &HWP,
        magika::ContentType::HxnfTroubleshoot => &HXNF_TROUBLESHOOT,
        magika::ContentType::HyperArchive => &HYPER_ARCHIVE,
        magika::ContentType::HyperplanningHyp => &HYPERPLANNING_HYP,
        magika::ContentType::Hyperwriter => &HYPERWRITER,
        magika::ContentType::HyteraFudx => &HYTERA_FUDX,
        magika::ContentType::Ibm3740 => &IBM_3740,
        magika::ContentType::IbmDisplaywrite => &IBM_DISPLAYWRITE,
        magika::ContentType::IbmMrnz => &IBM_MRNZ,
        magika::ContentType::IbmOs2ExtendedAttrEa => &IBM_OS2_EXTENDED_ATTR_EA,
        magika::ContentType::IbmOs2MkmsgfMsg => &IBM_OS2_MKMSGF_MSG,
        magika::ContentType::IbmSavedskf => &IBM_SAVEDSKF,
        magika::ContentType::IbmSavf => &IBM_SAVF,
        magika::ContentType::IbmStapCtl => &IBM_STAP_CTL,
        magika::ContentType::IbmTerse => &IBM_TERSE,
        magika::ContentType::IbmpcOvl => &IBMPC_OVL,
        magika::ContentType::IbusDict => &IBUS_DICT,
        magika::ContentType::Icc => &ICC,
        magika::ContentType::Icns => &ICNS,
        magika::ContentType::Ico => &ICO,
        magika::ContentType::Ics => &ICS,
        magika::ContentType::IcyTowerItr => &ICY_TOWER_ITR,
        magika::ContentType::IdRoq => &ID_ROQ,
        magika::ContentType::IdTed5 => &ID_TED5,
        magika::ContentType::Idatil => &IDATIL,
        magika::ContentType::Idb => &IDB,
        magika::ContentType::IdrKb => &IDR_KB,
        magika::ContentType::IdspSpr => &IDSP_SPR,
        magika::ContentType::IebcopyPds => &IEBCOPY_PDS,
        magika::ContentType::Ifd => &IFD,
        magika::ContentType::Iges => &IGES,
        magika::ContentType::Ignorefile => &IGNOREFILE,
        magika::ContentType::IgoPoiKns => &IGO_POI_KNS,
        magika::ContentType::Il2cppMetadata => &IL2CPP_METADATA,
        magika::ContentType::Ilbm => &ILBM,
        magika::ContentType::IllusionI3d => &ILLUSION_I3D,
        magika::ContentType::ImagesMusicSystem => &IMAGES_MUSIC_SYSTEM,
        magika::ContentType::ImagoImf => &IMAGO_IMF,
        magika::ContentType::Imd => &IMD,
        magika::ContentType::Img => &IMG,
        magika::ContentType::ImpArchive => &IMP_ARCHIVE,
        magika::ContentType::Imploder => &IMPLODER,
        magika::ContentType::ImuseMcmp => &IMUSE_MCMP,
        magika::ContentType::ImxSdmaFw => &IMX_SDMA_FW,
        magika::ContentType::InStereoIs => &IN_STEREO_IS,
        magika::ContentType::Indd => &INDD,
        magika::ContentType::InfocryptIcsl => &INFOCRYPT_ICSL,
        magika::ContentType::InfofileFlr => &INFOFILE_FLR,
        magika::ContentType::InfogramesIns => &INFOGRAMES_INS,
        magika::ContentType::InfotrieveTpx => &INFOTRIEVE_TPX,
        magika::ContentType::Ini => &INI,
        magika::ContentType::InmagicDbtext => &INMAGIC_DBTEXT,
        magika::ContentType::InnoSetup => &INNO_SETUP,
        magika::ContentType::InnodbTablespace => &INNODB_TABLESPACE,
        magika::ContentType::InstallerViseXyz => &INSTALLER_VISE_XYZ,
        magika::ContentType::InstallshieldIns => &INSTALLSHIELD_INS,
        magika::ContentType::InstallshieldInx => &INSTALLSHIELD_INX,
        magika::ContentType::InstallshieldIsu => &INSTALLSHIELD_ISU,
        magika::ContentType::InstallshieldPkg => &INSTALLSHIELD_PKG,
        magika::ContentType::InstarBneg => &INSTAR_BNEG,
        magika::ContentType::IntelCpdFirmware => &INTEL_CPD_FIRMWARE,
        magika::ContentType::IntelDvi => &INTEL_DVI,
        magika::ContentType::IntelEsifDv => &INTEL_ESIF_DV,
        magika::ContentType::IntelIscuFirmware => &INTEL_ISCU_FIRMWARE,
        magika::ContentType::IntelIvscFirmware => &INTEL_IVSC_FIRMWARE,
        magika::ContentType::IntelMeFpt => &INTEL_ME_FPT,
        magika::ContentType::Intelhex => &INTELHEX,
        magika::ContentType::Intellifont => &INTELLIFONT,
        magika::ContentType::InterleavedAdxAudioAix => &INTERLEAVED_ADX_AUDIO_AIX,
        magika::ContentType::Internetshortcut => &INTERNETSHORTCUT,
        magika::ContentType::InterplayAcm => &INTERPLAY_ACM,
        magika::ContentType::IonPmlDgm => &ION_PML_DGM,
        magika::ContentType::Iosapp => &IOSAPP,
        magika::ContentType::Ipynb => &IPYNB,
        magika::ContentType::Irlog1Log => &IRLOG1_LOG,
        magika::ContentType::Is3Archive => &IS3_ARCHIVE,
        magika::ContentType::IsIbt => &IS_IBT,
        magika::ContentType::Iscab => &ISCAB,
        magika::ContentType::IsiFracCar => &ISI_FRAC_CAR,
        magika::ContentType::IsiGmotor2Mas => &ISI_GMOTOR2_MAS,
        magika::ContentType::Iso => &ISO,
        magika::ContentType::IswlogSwl => &ISWLOG_SWL,
        magika::ContentType::Isz => &ISZ,
        magika::ContentType::It => &IT,
        magika::ContentType::Ivf => &IVF,
        magika::ContentType::Iw4xHeat => &IW4X_HEAT,
        magika::ContentType::IwlwifiUcode => &IWLWIFI_UCODE,
        magika::ContentType::IxIndex => &IX_INDEX,
        magika::ContentType::IxalanceIxs => &IXALANCE_IXS,
        magika::ContentType::J3d => &J3D,
        magika::ContentType::JackhammerJmf => &JACKHAMMER_JMF,
        magika::ContentType::JamPacker => &JAM_PACKER,
        magika::ContentType::JamcrackerJam => &JAMCRACKER_JAM,
        magika::ContentType::Jar => &JAR,
        magika::ContentType::JasonPageOld => &JASON_PAGE_OLD,
        magika::ContentType::Java => &JAVA,
        magika::ContentType::JavaWebstartIdx => &JAVA_WEBSTART_IDX,
        magika::ContentType::Javabytecode => &JAVABYTECODE,
        magika::ContentType::Javascript => &JAVASCRIPT,
        magika::ContentType::JaytraxJxs => &JAYTRAX_JXS,
        magika::ContentType::Jbig2 => &JBIG2,
        magika::ContentType::JbootSch2 => &JBOOT_SCH2,
        magika::ContentType::JeroenTelJt => &JEROEN_TEL_JT,
        magika::ContentType::JetbrainsPhmAt => &JETBRAINS_PHM_AT,
        magika::ContentType::Jffs2 => &JFFS2,
        magika::ContentType::Jfx1 => &JFX1,
        magika::ContentType::JgxpPackage => &JGXP_PACKAGE,
        magika::ContentType::Jif99a => &JIF99A,
        magika::ContentType::Jinja => &JINJA,
        magika::ContentType::Jks => &JKS,
        magika::ContentType::Jng => &JNG,
        magika::ContentType::Jnlp => &JNLP,
        magika::ContentType::JodaTz => &JODA_TZ,
        magika::ContentType::Jp2 => &JP2,
        magika::ContentType::Jp2Pgx => &JP2_PGX,
        magika::ContentType::Jpeg => &JPEG,
        magika::ContentType::JphpModule => &JPHP_MODULE,
        magika::ContentType::Json => &JSON,
        magika::ContentType::Jsonl => &JSONL,
        magika::ContentType::JsosMd8 => &JSOS_MD8,
        magika::ContentType::JuceProp => &JUCE_PROP,
        magika::ContentType::JuceVc2Preset => &JUCE_VC2_PRESET,
        magika::ContentType::Julia => &JULIA,
        magika::ContentType::JuliaJi => &JULIA_JI,
        magika::ContentType::JumbfC2pa => &JUMBF_C2PA,
        magika::ContentType::JvcEverioMoi => &JVC_EVERIO_MOI,
        magika::ContentType::JwlaunchP2 => &JWLAUNCH_P2,
        magika::ContentType::Jxl => &JXL,
        magika::ContentType::Jxs => &JXS,
        magika::ContentType::KapelMdf => &KAPEL_MDF,
        magika::ContentType::KasperskyUpds => &KASPERSKY_UPDS,
        magika::ContentType::KdRed => &KD_RED,
        magika::ContentType::KdeKwallet => &KDE_KWALLET,
        magika::ContentType::Keepass => &KEEPASS,
        magika::ContentType::KensAdlibKsm => &KENS_ADLIB_KSM,
        magika::ContentType::KensDigitalKdm => &KENS_DIGITAL_KDM,
        magika::ContentType::Keras => &KERAS,
        magika::ContentType::KerberosKirbi => &KERBEROS_KIRBI,
        magika::ContentType::Keychain => &KEYCHAIN,
        magika::ContentType::KeyenceLjLjhp => &KEYENCE_LJ_LJHP,
        magika::ContentType::Kgb => &KGB,
        magika::ContentType::KillLaKillDgkp => &KILL_LA_KILL_DGKP,
        magika::ContentType::KindleDrmion => &KINDLE_DRMION,
        magika::ContentType::KindleKfx => &KINDLE_KFX,
        magika::ContentType::KleiKtex => &KLEI_KTEX,
        magika::ContentType::Kml => &KML,
        magika::ContentType::Kmz => &KMZ,
        magika::ContentType::Koala => &KOALA,
        magika::ContentType::KodakDc20PresetPst => &KODAK_DC20_PRESET_PST,
        magika::ContentType::KodakPhotoCdInfoPcd => &KODAK_PHOTO_CD_INFO_PCD,
        magika::ContentType::KodiXbt => &KODI_XBT,
        magika::ContentType::KoeiLs11S11 => &KOEI_LS11_S11,
        magika::ContentType::KoeiTecmoG1t => &KOEI_TECMO_G1T,
        magika::ContentType::KoeiTecmoXws => &KOEI_TECMO_XWS,
        magika::ContentType::KohanSnd => &KOHAN_SND,
        magika::ContentType::Kompas3d => &KOMPAS_3D,
        magika::ContentType::KontaktHsinNki => &KONTAKT_HSIN_NKI,
        magika::ContentType::KontaktNcw => &KONTAKT_NCW,
        magika::ContentType::KontextLat => &KONTEXT_LAT,
        magika::ContentType::Kotlin => &KOTLIN,
        magika::ContentType::KritaTileData => &KRITA_TILE_DATA,
        magika::ContentType::KromMorkRes => &KROM_MORK_RES,
        magika::ContentType::KryofluxRaw => &KRYOFLUX_RAW,
        magika::ContentType::KspMu => &KSP_MU,
        magika::ContentType::Kss => &KSS,
        magika::ContentType::Kt => &KT,
        magika::ContentType::KudoCatalogDb => &KUDO_CATALOG_DB,
        magika::ContentType::KudoHashIndex => &KUDO_HASH_INDEX,
        magika::ContentType::KugouMusicKgm => &KUGOU_MUSIC_KGM,
        magika::ContentType::KvagAudio => &KVAG_AUDIO,
        magika::ContentType::Kwaj => &KWAJ,
        magika::ContentType::Kwikdraw => &KWIKDRAW,
        magika::ContentType::KyoceraRaw => &KYOCERA_RAW,
        magika::ContentType::LaAudio => &LA_AUDIO,
        magika::ContentType::LabgruppenIpxpreset => &LABGRUPPEN_IPXPRESET,
        magika::ContentType::LarianLsof => &LARIAN_LSOF,
        magika::ContentType::LarianLspk => &LARIAN_LSPK,
        magika::ContentType::Las => &LAS,
        magika::ContentType::Latex => &LATEX,
        magika::ContentType::LatticeBitstream => &LATTICE_BITSTREAM,
        magika::ContentType::LaxityPt => &LAXITY_PT,
        magika::ContentType::LayaairMeshLm => &LAYAAIR_MESH_LM,
        magika::ContentType::Lbr => &LBR,
        magika::ContentType::LcioSio => &LCIO_SIO,
        magika::ContentType::Ldb => &LDB,
        magika::ContentType::LeadtoolsCmp => &LEADTOOLS_CMP,
        magika::ContentType::LegglessLme => &LEGGLESS_LME,
        magika::ContentType::LegoAlpTun => &LEGO_ALP_TUN,
        magika::ContentType::LeicaApscFirmware => &LEICA_APSC_FIRMWARE,
        magika::ContentType::LeicaDmrFirmware => &LEICA_DMR_FIRMWARE,
        magika::ContentType::LeicaFirmware => &LEICA_FIRMWARE,
        magika::ContentType::LeicaLensDb => &LEICA_LENS_DB,
        magika::ContentType::LenovoLogdat => &LENOVO_LOGDAT,
        magika::ContentType::LeonardoImageFormatLeo => &LEONARDO_IMAGE_FORMAT_LEO,
        magika::ContentType::Leveldb => &LEVELDB,
        magika::ContentType::Lfm2Lfmat => &LFM2_LFMAT,
        magika::ContentType::Lha => &LHA,
        magika::ContentType::LibgdxG3db => &LIBGDX_G3DB,
        magika::ContentType::Liblzg => &LIBLZG,
        magika::ContentType::Lightwave => &LIGHTWAVE,
        magika::ContentType::Lineage2UnrealPkg => &LINEAGE2_UNREAL_PKG,
        magika::ContentType::LingeaLld => &LINGEA_LLD,
        magika::ContentType::LinuxKmodIndex => &LINUX_KMOD_INDEX,
        magika::ContentType::LinuxPerf => &LINUX_PERF,
        magika::ContentType::LinuxPsf => &LINUX_PSF,
        magika::ContentType::LionheartLion => &LIONHEART_LION,
        magika::ContentType::Liq => &LIQ,
        magika::ContentType::Lisp => &LISP,
        magika::ContentType::LithtechRez => &LITHTECH_REZ,
        magika::ContentType::Littlefs => &LITTLEFS,
        magika::ContentType::Live2dCmo3 => &LIVE2D_CMO3,
        magika::ContentType::Livecode => &LIVECODE,
        magika::ContentType::LlvmBitcode => &LLVM_BITCODE,
        magika::ContentType::LlvmIr => &LLVM_IR,
        magika::ContentType::LlvmProfdata => &LLVM_PROFDATA,
        magika::ContentType::Lmdb => &LMDB,
        magika::ContentType::Lnk => &LNK,
        magika::ContentType::LobMultipak => &LOB_MULTIPAK,
        magika::ContentType::LocaleStrndx => &LOCALE_STRNDX,
        magika::ContentType::Locoscript => &LOCOSCRIPT,
        magika::ContentType::LogitechCompress => &LOGITECH_COMPRESS,
        magika::ContentType::LoricielBnks => &LORICIEL_BNKS,
        magika::ContentType::Lotus123InstallDat => &LOTUS_123_INSTALL_DAT,
        magika::ContentType::LotusCmz => &LOTUS_CMZ,
        magika::ContentType::Lpac => &LPAC,
        magika::ContentType::Lrz => &LRZ,
        magika::ContentType::LsstatStat => &LSSTAT_STAT,
        magika::ContentType::Lua => &LUA,
        magika::ContentType::Luabytecode => &LUABYTECODE,
        magika::ContentType::LucasartsGob => &LUCASARTS_GOB,
        magika::ContentType::LucasartsLaa => &LUCASARTS_LAA,
        magika::ContentType::LucasartsSou => &LUCASARTS_SOU,
        magika::ContentType::LuceneSi => &LUCENE_SI,
        magika::ContentType::LurawaveLwf => &LURAWAVE_LWF,
        magika::ContentType::LuxonixPurityPth => &LUXONIX_PURITY_PTH,
        magika::ContentType::LytroLfp => &LYTRO_LFP,
        magika::ContentType::Lz => &LZ,
        magika::ContentType::Lz4 => &LZ4,
        magika::ContentType::Lzf => &LZF,
        magika::ContentType::Lzfse => &LZFSE,
        magika::ContentType::LzmCompressed => &LZM_COMPRESSED,
        magika::ContentType::Lzop => &LZOP,
        magika::ContentType::Lzp1 => &LZP1,
        magika::ContentType::Lzsa => &LZSA,
        magika::ContentType::Lzx => &LZX,
        magika::ContentType::M3u => &M3U,
        magika::ContentType::M4 => &M4,
        magika::ContentType::MacWord => &MAC_WORD,
        magika::ContentType::Macho => &MACHO,
        magika::ContentType::MacosFsevents => &MACOS_FSEVENTS,
        magika::ContentType::MacosUtmpx => &MACOS_UTMPX,
        magika::ContentType::MacroExpressMex => &MACRO_EXPRESS_MEX,
        magika::ContentType::Macwrite2 => &MACWRITE2,
        magika::ContentType::Maff => &MAFF,
        magika::ContentType::MagicavoxelVox => &MAGICAVOXEL_VOX,
        magika::ContentType::MagneticScrollsGfx => &MAGNETIC_SCROLLS_GFX,
        magika::ContentType::MagneticScrollsMag => &MAGNETIC_SCROLLS_MAG,
        magika::ContentType::Makefile => &MAKEFILE,
        magika::ContentType::MalcatCsig => &MALCAT_CSIG,
        magika::ContentType::Mandelbulb3dM3p => &MANDELBULB3D_M3P,
        magika::ContentType::Manuscript => &MANUSCRIPT,
        magika::ContentType::Mapinfo => &MAPINFO,
        magika::ContentType::MapsforgeBinaryVectorMap => &MAPSFORGE_BINARY_VECTOR_MAP,
        magika::ContentType::MarArchiveMac => &MAR_ARCHIVE_MAC,
        magika::ContentType::MarinerWrite => &MARINER_WRITE,
        magika::ContentType::Mark2Mk2 => &MARK2_MK2,
        magika::ContentType::MarkCooksey => &MARK_COOKSEY,
        magika::ContentType::Markdown => &MARKDOWN,
        magika::ContentType::MarmaladeDz => &MARMALADE_DZ,
        magika::ContentType::MarvellCptUcode => &MARVELL_CPT_UCODE,
        magika::ContentType::Mas => &MAS,
        magika::ContentType::MasterTracksMts => &MASTER_TRACKS_MTS,
        magika::ContentType::MastercamV9Mc9 => &MASTERCAM_V9_MC9,
        magika::ContentType::MastertrackerMtr => &MASTERTRACKER_MTR,
        magika::ContentType::Mat => &MAT,
        magika::ContentType::Matlab => &MATLAB,
        magika::ContentType::MatterOta => &MATTER_OTA,
        magika::ContentType::Max => &MAX,
        magika::ContentType::Maya => &MAYA,
        magika::ContentType::Mbms => &MBMS,
        magika::ContentType::Mbtiles => &MBTILES,
        magika::ContentType::Mcap => &MCAP,
        magika::ContentType::McftCft => &MCFT_CFT,
        magika::ContentType::Mcmd => &MCMD,
        magika::ContentType::McsStimulusStm => &MCS_STIMULUS_STM,
        magika::ContentType::McubootImg => &MCUBOOT_IMG,
        magika::ContentType::Md2 => &MD2,
        magika::ContentType::Mdcd => &MDCD,
        magika::ContentType::MdlDigitrakker => &MDL_DIGITRAKKER,
        magika::ContentType::Mds => &MDS,
        magika::ContentType::MdxelmG2p => &MDXELM_G2P,
        magika::ContentType::MediatekBrlyt => &MEDIATEK_BRLYT,
        magika::ContentType::MediatekBtFw => &MEDIATEK_BT_FW,
        magika::ContentType::MediatekImage => &MEDIATEK_IMAGE,
        magika::ContentType::MediatekPreloader => &MEDIATEK_PRELOADER,
        magika::ContentType::MediatekSignedImg => &MEDIATEK_SIGNED_IMG,
        magika::ContentType::MedleyMso => &MEDLEY_MSO,
        magika::ContentType::MegastationMsm => &MEGASTATION_MSM,
        magika::ContentType::MegatrackerMgt => &MEGATRACKER_MGT,
        magika::ContentType::MegazeuxWorld => &MEGAZEUX_WORLD,
        magika::ContentType::MeleeSlippi => &MELEE_SLIPPI,
        magika::ContentType::MercedesCff => &MERCEDES_CFF,
        magika::ContentType::Metakit => &METAKIT,
        magika::ContentType::Metallib => &METALLIB,
        magika::ContentType::MetatraderEx4 => &METATRADER_EX4,
        magika::ContentType::MetatraderEx5 => &METATRADER_EX5,
        magika::ContentType::MetatraderHc => &METATRADER_HC,
        magika::ContentType::MetatraderSrv => &METATRADER_SRV,
        magika::ContentType::Mgsdrv => &MGSDRV,
        magika::ContentType::Mht => &MHT,
        magika::ContentType::MicrochipMpfs => &MICROCHIP_MPFS,
        magika::ContentType::MicrofocusGnt => &MICROFOCUS_GNT,
        magika::ContentType::MicrohouseProsecBin => &MICROHOUSE_PROSEC_BIN,
        magika::ContentType::MicromeriticsSubset => &MICROMERITICS_SUBSET,
        magika::ContentType::Midi => &MIDI,
        magika::ContentType::Miff => &MIFF,
        magika::ContentType::MikrotikNpk => &MIKROTIK_NPK,
        magika::ContentType::MilestonesMlb => &MILESTONES_MLB,
        magika::ContentType::MindwareMfs => &MINDWARE_MFS,
        magika::ContentType::Minidump => &MINIDUMP,
        magika::ContentType::MinitaurLogProto => &MINITAUR_LOG_PROTO,
        magika::ContentType::MirMap => &MIR_MAP,
        magika::ContentType::MkjamzMkj => &MKJAMZ_MKJ,
        magika::ContentType::Mkv => &MKV,
        magika::ContentType::MlatMad => &MLAT_MAD,
        magika::ContentType::MlocateDb => &MLOCATE_DB,
        magika::ContentType::MmGeov1R565 => &MM_GEOV1_R565,
        magika::ContentType::Mmcmp => &MMCMP,
        magika::ContentType::MmdPmd => &MMD_PMD,
        magika::ContentType::MmdPmx => &MMD_PMX,
        magika::ContentType::MmdVmd => &MMD_VMD,
        magika::ContentType::Mo3 => &MO3,
        magika::ContentType::MobicoreMclf => &MOBICORE_MCLF,
        magika::ContentType::MomGrf => &MOM_GRF,
        magika::ContentType::MonotoneMon => &MONOTONE_MON,
        magika::ContentType::Moof => &MOOF,
        magika::ContentType::MoonblasterMbm => &MOONBLASTER_MBM,
        magika::ContentType::MoreOutline => &MORE_OUTLINE,
        magika::ContentType::MoshidrawYz => &MOSHIDRAW_YZ,
        magika::ContentType::MotorolaMotologo => &MOTOROLA_MOTOLOGO,
        magika::ContentType::MotorolaPivImg => &MOTOROLA_PIV_IMG,
        magika::ContentType::MousewriteDoc => &MOUSEWRITE_DOC,
        magika::ContentType::MoxaFrm => &MOXA_FRM,
        magika::ContentType::Mozlz4 => &MOZLZ4,
        magika::ContentType::Mp2 => &MP2,
        magika::ContentType::Mp3 => &MP3,
        magika::ContentType::Mp4 => &MP4,
        magika::ContentType::Mpegts => &MPEGTS,
        magika::ContentType::Mpq => &MPQ,
        magika::ContentType::Mpu401Mtk => &MPU401_MTK,
        magika::ContentType::MrcMap => &MRC_MAP,
        magika::ContentType::MrubyRiteMrb => &MRUBY_RITE_MRB,
        magika::ContentType::MsAdvisorHlp => &MS_ADVISOR_HLP,
        magika::ContentType::MsAnswerWizardAw => &MS_ANSWER_WIZARD_AW,
        magika::ContentType::MsCertStoreSst => &MS_CERT_STORE_SST,
        magika::ContentType::MsNetmonNpb => &MS_NETMON_NPB,
        magika::ContentType::MsPrague => &MS_PRAGUE,
        magika::ContentType::MsPublisherBorderartBdr => &MS_PUBLISHER_BORDERART_BDR,
        magika::ContentType::MsPublisherPagewizardPoc => &MS_PUBLISHER_PAGEWIZARD_POC,
        magika::ContentType::MsPublisherPreviewPrv => &MS_PUBLISHER_PREVIEW_PRV,
        magika::ContentType::MsSetupTdf => &MS_SETUP_TDF,
        magika::ContentType::MsWindowsBootLogoLgo => &MS_WINDOWS_BOOT_LOGO_LGO,
        magika::ContentType::MsWindowsPostscriptWpd => &MS_WINDOWS_POSTSCRIPT_WPD,
        magika::ContentType::Mscompress => &MSCOMPRESS,
        magika::ContentType::Msdelta => &MSDELTA,
        magika::ContentType::MsdosCountrySys => &MSDOS_COUNTRY_SYS,
        magika::ContentType::MsdosKeyboardSys => &MSDOS_KEYBOARD_SYS,
        magika::ContentType::MsfsBgl => &MSFS_BGL,
        magika::ContentType::MsfsSpb => &MSFS_SPB,
        magika::ContentType::Msgpack => &MSGPACK,
        magika::ContentType::Msi => &MSI,
        magika::ContentType::MsieIndexDat => &MSIE_INDEX_DAT,
        magika::ContentType::Msix => &MSIX,
        magika::ContentType::MspPaint => &MSP_PAINT,
        magika::ContentType::MssscardAxLog => &MSSSCARD_AX_LOG,
        magika::ContentType::Mst => &MST,
        magika::ContentType::MstorIbx => &MSTOR_IBX,
        magika::ContentType::MstsSimis => &MSTS_SIMIS,
        magika::ContentType::MsvcIfc => &MSVC_IFC,
        magika::ContentType::MsvcIpdb => &MSVC_IPDB,
        magika::ContentType::MsworksWdb => &MSWORKS_WDB,
        magika::ContentType::MsxCas => &MSX_CAS,
        magika::ContentType::MsxFacMus => &MSX_FAC_MUS,
        magika::ContentType::MsxMpk => &MSX_MPK,
        magika::ContentType::Mt2 => &MT2,
        magika::ContentType::MtcChiptune => &MTC_CHIPTUNE,
        magika::ContentType::Mtl => &MTL,
        magika::ContentType::Multimate => &MULTIMATE,
        magika::ContentType::Multiplan => &MULTIPLAN,
        magika::ContentType::Mum => &MUM,
        magika::ContentType::MusicStudioSng => &MUSIC_STUDIO_SNG,
        magika::ContentType::MusicalEnlightenmentTme => &MUSICAL_ENLIGHTENMENT_TME,
        magika::ContentType::MusiclineMl => &MUSICLINE_ML,
        magika::ContentType::MvsObj => &MVS_OBJ,
        magika::ContentType::MvsTracker => &MVS_TRACKER,
        magika::ContentType::MvxMvm => &MVX_MVM,
        magika::ContentType::Mxmf => &MXMF,
        magika::ContentType::Mxtx => &MXTX,
        magika::ContentType::MysqlStorage => &MYSQL_STORAGE,
        magika::ContentType::N64Rom => &N64_ROM,
        magika::ContentType::NafBio => &NAF_BIO,
        magika::ContentType::NamcoHbss => &NAMCO_HBSS,
        magika::ContentType::NamxProfile => &NAMX_PROFILE,
        magika::ContentType::NasmRdoff => &NASM_RDOFF,
        magika::ContentType::Nba2kCmg => &NBA2K_CMG,
        magika::ContentType::Nbi => &NBI,
        magika::ContentType::NcMsg => &NC_MSG,
        magika::ContentType::NcstudioDyn => &NCSTUDIO_DYN,
        magika::ContentType::Ndif => &NDIF,
        magika::ContentType::NdsNsbtx => &NDS_NSBTX,
        magika::ContentType::Ned => &NED,
        magika::ContentType::NeogeoAbkcpurSrm => &NEOGEO_ABKCPUR_SRM,
        magika::ContentType::NesRom => &NES_ROM,
        magika::ContentType::Netcdf => &NETCDF,
        magika::ContentType::NetgearChk => &NETGEAR_CHK,
        magika::ContentType::NetronNfw => &NETRON_NFW,
        magika::ContentType::NewwordOvr => &NEWWORD_OVR,
        magika::ContentType::NexonKartrider1s => &NEXON_KARTRIDER_1S,
        magika::ContentType::NextstepWmd => &NEXTSTEP_WMD,
        magika::ContentType::NexusImgdata => &NEXUS_IMGDATA,
        magika::ContentType::NfdumpFlow => &NFDUMP_FLOW,
        magika::ContentType::NgpRom => &NGP_ROM,
        magika::ContentType::NibtoolsNib => &NIBTOOLS_NIB,
        magika::ContentType::Nif => &NIF,
        magika::ContentType::Nifti => &NIFTI,
        magika::ContentType::NintendoAamp => &NINTENDO_AAMP,
        magika::ContentType::NintendoAinb => &NINTENDO_AINB,
        magika::ContentType::NintendoAst => &NINTENDO_AST,
        magika::ContentType::NintendoBmg => &NINTENDO_BMG,
        magika::ContentType::NintendoBntx => &NINTENDO_BNTX,
        magika::ContentType::NintendoBwav => &NINTENDO_BWAV,
        magika::ContentType::NintendoByml => &NINTENDO_BYML,
        magika::ContentType::NintendoFont => &NINTENDO_FONT,
        magika::ContentType::NintendoMsbt => &NINTENDO_MSBT,
        magika::ContentType::NintendoNarc => &NINTENDO_NARC,
        magika::ContentType::NintendoU8 => &NINTENDO_U8,
        magika::ContentType::NintendoWiiBnr => &NINTENDO_WII_BNR,
        magika::ContentType::NintendoWiiuGtx => &NINTENDO_WIIU_GTX,
        magika::ContentType::NistSphere => &NIST_SPHERE,
        magika::ContentType::NitroplusNpa => &NITROPLUS_NPA,
        magika::ContentType::NmsMbin => &NMS_MBIN,
        magika::ContentType::NokiaBb5Ppm => &NOKIA_BB5_PPM,
        magika::ContentType::NokiaNgg => &NOKIA_NGG,
        magika::ContentType::NokiaNlm => &NOKIA_NLM,
        magika::ContentType::NokiaNpm => &NOKIA_NPM,
        magika::ContentType::NortonGuidesHlp => &NORTON_GUIDES_HLP,
        magika::ContentType::NosInstallerData => &NOS_INSTALLER_DATA,
        magika::ContentType::NotatorSong => &NOTATOR_SONG,
        magika::ContentType::NovellNetwareMessageMsg => &NOVELL_NETWARE_MESSAGE_MSG,
        magika::ContentType::NovellNetwareRplRom => &NOVELL_NETWARE_RPL_ROM,
        magika::ContentType::NovellSysHelpHep => &NOVELL_SYS_HELP_HEP,
        magika::ContentType::Npy => &NPY,
        magika::ContentType::Npz => &NPZ,
        magika::ContentType::Nrrd => &NRRD,
        magika::ContentType::Nsf => &NSF,
        magika::ContentType::Nsis => &NSIS,
        magika::ContentType::NtfscloneImg => &NTFSCLONE_IMG,
        magika::ContentType::NtspSystem => &NTSP_SYSTEM,
        magika::ContentType::Nupkg => &NUPKG,
        magika::ContentType::Nuppelvideo => &NUPPELVIDEO,
        magika::ContentType::NutVideo => &NUT_VIDEO,
        magika::ContentType::ObarskiOsp => &OBARSKI_OSP,
        magika::ContentType::OberonOdc => &OBERON_ODC,
        magika::ContentType::Obj => &OBJ,
        magika::ContentType::Objectivec => &OBJECTIVEC,
        magika::ContentType::Ocaml => &OCAML,
        magika::ContentType::OcamlCmi => &OCAML_CMI,
        magika::ContentType::Ocdbt => &OCDBT,
        magika::ContentType::Odex => &ODEX,
        magika::ContentType::Odp => &ODP,
        magika::ContentType::Ods => &ODS,
        magika::ContentType::Odt => &ODT,
        magika::ContentType::OfficeOwnerLock => &OFFICE_OWNER_LOCK,
        magika::ContentType::Ofr => &OFR,
        magika::ContentType::Ogg => &OGG,
        magika::ContentType::Ogl => &OGL,
        magika::ContentType::OgreMesh => &OGRE_MESH,
        magika::ContentType::OmegaLogicCork => &OMEGA_LOGIC_CORK,
        magika::ContentType::Omf => &OMF,
        magika::ContentType::OmfOnyx => &OMF_ONYX,
        magika::ContentType::One => &ONE,
        magika::ContentType::OnedriveOdl => &ONEDRIVE_ODL,
        magika::ContentType::Onetoc2 => &ONETOC2,
        magika::ContentType::Onnx => &ONNX,
        magika::ContentType::Openfst => &OPENFST,
        magika::ContentType::Openrct2Park => &OPENRCT2_PARK,
        magika::ContentType::OpensshKrl => &OPENSSH_KRL,
        magika::ContentType::OpenvinoIr => &OPENVINO_IR,
        magika::ContentType::OpenzimZim => &OPENZIM_ZIM,
        magika::ContentType::Opx => &OPX,
        magika::ContentType::OracleDb => &ORACLE_DB,
        magika::ContentType::Orc => &ORC,
        magika::ContentType::Org => &ORG,
        magika::ContentType::OricDsk => &ORIC_DSK,
        magika::ContentType::Os2Font => &OS2_FONT,
        magika::ContentType::Os2Ftcomp => &OS2_FTCOMP,
        magika::ContentType::Os2Inf => &OS2_INF,
        magika::ContentType::Os2Pack => &OS2_PACK,
        magika::ContentType::OsaskHrbExec => &OSASK_HRB_EXEC,
        magika::ContentType::OsgiFrameworkInfo4 => &OSGI_FRAMEWORK_INFO4,
        magika::ContentType::OslOsoBinary => &OSL_OSO_BINARY,
        magika::ContentType::Osm => &OSM,
        magika::ContentType::Otf => &OTF,
        magika::ContentType::Outlook => &OUTLOOK,
        magika::ContentType::OutlookNk2 => &OUTLOOK_NK2,
        magika::ContentType::Pablopaint => &PABLOPAINT,
        magika::ContentType::PackIce => &PACK_ICE,
        magika::ContentType::PadsPcb => &PADS_PCB,
        magika::ContentType::Paint4d => &PAINT4D,
        magika::ContentType::PalladixPcs => &PALLADIX_PCS,
        magika::ContentType::Palmos => &PALMOS,
        magika::ContentType::PanasonicS3d => &PANASONIC_S3D,
        magika::ContentType::PanasonicUpdFirmware => &PANASONIC_UPD_FIRMWARE,
        magika::ContentType::PangolinPrr => &PANGOLIN_PRR,
        magika::ContentType::PaqArchive => &PAQ_ARCHIVE,
        magika::ContentType::Paradox => &PARADOX,
        magika::ContentType::ParadoxMesh => &PARADOX_MESH,
        magika::ContentType::ParallelsHdd => &PARALLELS_HDD,
        magika::ContentType::Parchive => &PARCHIVE,
        magika::ContentType::Parquet => &PARQUET,
        magika::ContentType::ParsonsWrk => &PARSONS_WRK,
        magika::ContentType::Pascal => &PASCAL,
        magika::ContentType::PassmarkPt => &PASSMARK_PT,
        magika::ContentType::Passwordsafe => &PASSWORDSAFE,
        magika::ContentType::PaulRobothamDat => &PAUL_ROBOTHAM_DAT,
        magika::ContentType::PazligsawPzl => &PAZLIGSAW_PZL,
        magika::ContentType::Pbm => &PBM,
        magika::ContentType::Pbp => &PBP,
        magika::ContentType::Pc88T88 => &PC88_T88,
        magika::ContentType::Pcap => &PCAP,
        magika::ContentType::Pcapng => &PCAPNG,
        magika::ContentType::PcboardPpe => &PCBOARD_PPE,
        magika::ContentType::Pcd => &PCD,
        magika::ContentType::Pcf => &PCF,
        magika::ContentType::PcrFont => &PCR_FONT,
        magika::ContentType::PctoolsMsg => &PCTOOLS_MSG,
        magika::ContentType::PcxLibrary => &PCX_LIBRARY,
        magika::ContentType::Pdb => &PDB,
        magika::ContentType::Pdf => &PDF,
        magika::ContentType::PdfannotatorSil => &PDFANNOTATOR_SIL,
        magika::ContentType::Pdi2Dictionary => &PDI2_DICTIONARY,
        magika::ContentType::Pdn => &PDN,
        magika::ContentType::Pdp10Failsafe => &PDP10_FAILSAFE,
        magika::ContentType::Pdp11Lda => &PDP11_LDA,
        magika::ContentType::Pdp8Sv => &PDP8_SV,
        magika::ContentType::PdseProg => &PDSE_PROG,
        magika::ContentType::Pea => &PEA,
        magika::ContentType::Pebin => &PEBIN,
        magika::ContentType::Pem => &PEM,
        magika::ContentType::PepakuraPdi1 => &PEPAKURA_PDI1,
        magika::ContentType::PerfJitdump => &PERF_JITDUMP,
        magika::ContentType::Perl => &PERL,
        magika::ContentType::PerqSeg => &PERQ_SEG,
        magika::ContentType::Pfm => &PFM,
        magika::ContentType::Pfpart3 => &PFPART3,
        magika::ContentType::Pfr => &PFR,
        magika::ContentType::Pgf => &PGF,
        magika::ContentType::Pgp => &PGP,
        magika::ContentType::PgpLocstr => &PGP_LOCSTR,
        magika::ContentType::PharoFuel => &PHARO_FUEL,
        magika::ContentType::Phm => &PHM,
        magika::ContentType::PhotoshopAbr => &PHOTOSHOP_ABR,
        magika::ContentType::Php => &PHP,
        magika::ContentType::PiaproNtdb => &PIAPRO_NTDB,
        magika::ContentType::PicasaYtf => &PICASA_YTF,
        magika::ContentType::Pickle => &PICKLE,
        magika::ContentType::PierreAdanePap => &PIERRE_ADANE_PAP,
        magika::ContentType::PismoCiso => &PISMO_CISO,
        magika::ContentType::PixelPaintersFmf => &PIXEL_PAINTERS_FMF,
        magika::ContentType::PixelPaintersFtf => &PIXEL_PAINTERS_FTF,
        magika::ContentType::PixinsightXisf => &PIXINSIGHT_XISF,
        magika::ContentType::Pkcs15Ky => &PKCS15_KY,
        magika::ContentType::Pkm => &PKM,
        magika::ContentType::PlatoLesson => &PLATO_LESSON,
        magika::ContentType::PlayerproMadg => &PLAYERPRO_MADG,
        magika::ContentType::PlextalkProjectFileImdn => &PLEXTALK_PROJECT_FILE_IMDN,
        magika::ContentType::Plm => &PLM,
        magika::ContentType::Ply => &PLY,
        magika::ContentType::PmPicture => &PM_PICTURE,
        magika::ContentType::PmcCruncher => &PMC_CRUNCHER,
        magika::ContentType::PmdM => &PMD_M,
        magika::ContentType::Pmtiles => &PMTILES,
        magika::ContentType::Png => &PNG,
        magika::ContentType::Po => &PO,
        magika::ContentType::Pod5 => &POD5,
        magika::ContentType::PodbotPwf => &PODBOT_PWF,
        magika::ContentType::PokemonGarc => &POKEMON_GARC,
        magika::ContentType::PokemonGfpak => &POKEMON_GFPAK,
        magika::ContentType::PopPlv => &POP_PLV,
        magika::ContentType::PopcapPam => &POPCAP_PAM,
        magika::ContentType::PopstarterVcd => &POPSTARTER_VCD,
        magika::ContentType::PortableDatabasePdb => &PORTABLE_DATABASE_PDB,
        magika::ContentType::PortableFormFilePff => &PORTABLE_FORM_FILE_PFF,
        magika::ContentType::PostgresDump => &POSTGRES_DUMP,
        magika::ContentType::Postscript => &POSTSCRIPT,
        magika::ContentType::PowerbuilderPbd => &POWERBUILDER_PBD,
        magika::ContentType::PowerbuilderPsr => &POWERBUILDER_PSR,
        magika::ContentType::PowerisoDaa => &POWERISO_DAA,
        magika::ContentType::Powerpacker => &POWERPACKER,
        magika::ContentType::Powershell => &POWERSHELL,
        magika::ContentType::PowervrObjectDataPod => &POWERVR_OBJECT_DATA_POD,
        magika::ContentType::PpfPatch => &PPF_PATCH,
        magika::ContentType::PpmMpd => &PPM_MPD,
        magika::ContentType::Ppt => &PPT,
        magika::ContentType::Pptx => &PPTX,
        magika::ContentType::PremierePrmi => &PREMIERE_PRMI,
        magika::ContentType::PresetSap => &PRESET_SAP,
        magika::ContentType::PretrackerPrt => &PRETRACKER_PRT,
        magika::ContentType::Printfox => &PRINTFOX,
        magika::ContentType::PristonTaleSmd => &PRISTON_TALE_SMD,
        magika::ContentType::ProcmonPmc => &PROCMON_PMC,
        magika::ContentType::Project64Pj => &PROJECT64_PJ,
        magika::ContentType::Prolog => &PROLOG,
        magika::ContentType::Proshape => &PROSHAPE,
        magika::ContentType::Proteindb => &PROTEINDB,
        magika::ContentType::ProteusIsisDsn => &PROTEUS_ISIS_DSN,
        magika::ContentType::ProteusPdspart => &PROTEUS_PDSPART,
        magika::ContentType::Proto => &PROTO,
        magika::ContentType::Protobuf => &PROTOBUF,
        magika::ContentType::ProtrekkrPtk => &PROTREKKR_PTK,
        magika::ContentType::ProxUsbXha => &PROX_USB_XHA,
        magika::ContentType::ProxmoxPbs => &PROXMOX_PBS,
        magika::ContentType::PrvwPrw => &PRVW_PRW,
        magika::ContentType::Ps1Mcr => &PS1_MCR,
        magika::ContentType::Ps2Hd => &PS2_HD,
        magika::ContentType::Ps2Ipu => &PS2_IPU,
        magika::ContentType::Ps2Vmc => &PS2_VMC,
        magika::ContentType::Ps3Pkg => &PS3_PKG,
        magika::ContentType::Ps4Pkg => &PS4_PKG,
        magika::ContentType::PsSfo => &PS_SFO,
        magika::ContentType::PsaModule => &PSA_MODULE,
        magika::ContentType::Psc => &PSC,
        magika::ContentType::Psd => &PSD,
        magika::ContentType::Psf => &PSF,
        magika::ContentType::PsgSleep => &PSG_SLEEP,
        magika::ContentType::Psion3Word => &PSION3_WORD,
        magika::ContentType::PsionOpl => &PSION_OPL,
        magika::ContentType::PsionSiboApp => &PSION_SIBO_APP,
        magika::ContentType::PsmMasi => &PSM_MASI,
        magika::ContentType::PspDax => &PSP_DAX,
        magika::ContentType::PsxExe => &PSX_EXE,
        magika::ContentType::Psy => &PSY,
        magika::ContentType::Ptex => &PTEX,
        magika::ContentType::Ptf => &PTF,
        magika::ContentType::Ptm => &PTM,
        magika::ContentType::Pub => &PUB,
        magika::ContentType::PuffinPatch => &PUFFIN_PATCH,
        magika::ContentType::Pumatracker => &PUMATRACKER,
        magika::ContentType::Pvi => &PVI,
        magika::ContentType::Pvm => &PVM,
        magika::ContentType::Pvr => &PVR,
        magika::ContentType::PxhgPackage => &PXHG_PACKAGE,
        magika::ContentType::Pxtone => &PXTONE,
        magika::ContentType::Python => &PYTHON,
        magika::ContentType::Pythonbytecode => &PYTHONBYTECODE,
        magika::ContentType::Pythonpar => &PYTHONPAR,
        magika::ContentType::Pytorch => &PYTORCH,
        magika::ContentType::PytorchExport => &PYTORCH_EXPORT,
        magika::ContentType::QAWordProcessor => &Q_A_WORD_PROCESSOR,
        magika::ContentType::QcaAthFirmware => &QCA_ATH_FIRMWARE,
        magika::ContentType::QcaBtTlv => &QCA_BT_TLV,
        magika::ContentType::Qcdt => &QCDT,
        magika::ContentType::Qcow2 => &QCOW2,
        magika::ContentType::QemuQed => &QEMU_QED,
        magika::ContentType::Qgis => &QGIS,
        magika::ContentType::QnxDeflate => &QNX_DEFLATE,
        magika::ContentType::QnxIfs => &QNX_IFS,
        magika::ContentType::Qoi => &QOI,
        magika::ContentType::Qt => &QT,
        magika::ContentType::Quake3Qvm => &QUAKE3_QVM,
        magika::ContentType::QuakeMdl => &QUAKE_MDL,
        magika::ContentType::QualcommAcdb => &QUALCOMM_ACDB,
        magika::ContentType::QualcommFbpk => &QUALCOMM_FBPK,
        magika::ContentType::QualcommMetaImage => &QUALCOMM_META_IMAGE,
        magika::ContentType::QualcommMibib => &QUALCOMM_MIBIB,
        magika::ContentType::QualcommSbl1 => &QUALCOMM_SBL1,
        magika::ContentType::QualcommSplash => &QUALCOMM_SPLASH,
        magika::ContentType::QuartetSet => &QUARTET_SET,
        magika::ContentType::QuartetSt => &QUARTET_ST,
        magika::ContentType::QuartusPof => &QUARTUS_POF,
        magika::ContentType::QuattroProWb1 => &QUATTRO_PRO_WB1,
        magika::ContentType::QuestCas => &QUEST_CAS,
        magika::ContentType::Quicken3DatabaseFileQst => &QUICKEN_3_DATABASE_FILE_QST,
        magika::ContentType::QuicklinkQfx => &QUICKLINK_QFX,
        magika::ContentType::R => &R,
        magika::ContentType::RacerDof1 => &RACER_DOF1,
        magika::ContentType::RadBinkAudio => &RAD_BINK_AUDIO,
        magika::ContentType::RaddbgRdi => &RADDBG_RDI,
        magika::ContentType::RadicalRcf => &RADICAL_RCF,
        magika::ContentType::RadicalRsd => &RADICAL_RSD,
        magika::ContentType::RadiocefRcf => &RADIOCEF_RCF,
        magika::ContentType::RageAwc => &RAGE_AWC,
        magika::ContentType::RageRsc7 => &RAGE_RSC7,
        magika::ContentType::Ragtime => &RAGTIME,
        magika::ContentType::RailworksDav => &RAILWORKS_DAV,
        magika::ContentType::RailworksSerz => &RAILWORKS_SERZ,
        magika::ContentType::RamtrackerTrk => &RAMTRACKER_TRK,
        magika::ContentType::RanOnlineIms => &RAN_ONLINE_IMS,
        magika::ContentType::Randombytes => &RANDOMBYTES,
        magika::ContentType::Randomtxt => &RANDOMTXT,
        magika::ContentType::RapiscanRcf => &RAPISCAN_RCF,
        magika::ContentType::Rar => &RAR,
        magika::ContentType::Rarc => &RARC,
        magika::ContentType::RavenXbnfBnx => &RAVEN_XBNF_BNX,
        magika::ContentType::RcbotRcw => &RCBOT_RCW,
        magika::ContentType::Rdata => &RDATA,
        magika::ContentType::Rdf => &RDF,
        magika::ContentType::Rdp => &RDP,
        magika::ContentType::Re4Mdt => &RE4_MDT,
        magika::ContentType::ReEngineChain => &RE_ENGINE_CHAIN,
        magika::ContentType::ReEngineMcamlist => &RE_ENGINE_MCAMLIST,
        magika::ContentType::ReEngineMdf2 => &RE_ENGINE_MDF2,
        magika::ContentType::RealbasicRbbf => &REALBASIC_RBBF,
        magika::ContentType::Realm => &REALM,
        magika::ContentType::Realmedia => &REALMEDIA,
        magika::ContentType::RealtekBtFw => &REALTEK_BT_FW,
        magika::ContentType::RealtekCsys => &REALTEK_CSYS,
        magika::ContentType::RealtekRtk0 => &REALTEK_RTK0,
        magika::ContentType::RealtekWifiFw => &REALTEK_WIFI_FW,
        magika::ContentType::RebellionAsura => &REBELLION_ASURA,
        magika::ContentType::RecomposerRcp => &RECOMPOSER_RCP,
        magika::ContentType::Redb => &REDB,
        magika::ContentType::RedcodeR3d => &REDCODE_R3D,
        magika::ContentType::RedengineRdar => &REDENGINE_RDAR,
        magika::ContentType::RedisAof => &REDIS_AOF,
        magika::ContentType::RedisRdb => &REDIS_RDB,
        magika::ContentType::Redthumb => &REDTHUMB,
        magika::ContentType::RefpackBig => &REFPACK_BIG,
        magika::ContentType::RekoCardset => &REKO_CARDSET,
        magika::ContentType::RelicArchive => &RELIC_ARCHIVE,
        magika::ContentType::RelicRbh => &RELIC_RBH,
        magika::ContentType::RescsegCmf => &RESCSEG_CMF,
        magika::ContentType::ResfileRsc => &RESFILE_RSC,
        magika::ContentType::RfzpackRfz => &RFZPACK_RFZ,
        magika::ContentType::RgmhSave => &RGMH_SAVE,
        magika::ContentType::Rgssad => &RGSSAD,
        magika::ContentType::RhideGdt => &RHIDE_GDT,
        magika::ContentType::Rhinoceros => &RHINOCEROS,
        magika::ContentType::Riegeli => &RIEGELI,
        magika::ContentType::RillsoftRpj => &RILLSOFT_RPJ,
        magika::ContentType::RiscosFont => &RISCOS_FONT,
        magika::ContentType::RiveRiv => &RIVE_RIV,
        magika::ContentType::Rjp => &RJP,
        magika::ContentType::RkaAudio => &RKA_AUDIO,
        magika::ContentType::Rli => &RLI,
        magika::ContentType::Rlib => &RLIB,
        magika::ContentType::Rnc => &RNC,
        magika::ContentType::Rns => &RNS,
        magika::ContentType::RobHubbardRho => &ROB_HUBBARD_RHO,
        magika::ContentType::RoboplayPly => &ROBOPLAY_PLY,
        magika::ContentType::RockchipRkaf => &ROCKCHIP_RKAF,
        magika::ContentType::RockchipRkfw => &ROCKCHIP_RKFW,
        magika::ContentType::RockchipRsce => &ROCKCHIP_RSCE,
        magika::ContentType::Rocksdb => &ROCKSDB,
        magika::ContentType::RockstarRpf => &ROCKSTAR_RPF,
        magika::ContentType::RockyRip => &ROCKY_RIP,
        magika::ContentType::RolandSvq => &ROLAND_SVQ,
        magika::ContentType::Romfs => &ROMFS,
        magika::ContentType::RosBagFormat => &ROS_BAG_FORMAT,
        magika::ContentType::RouterWlkConfig => &ROUTER_WLK_CONFIG,
        magika::ContentType::Rpa => &RPA,
        magika::ContentType::RpgWolf => &RPG_WOLF,
        magika::ContentType::RpgmakerXyz => &RPGMAKER_XYZ,
        magika::ContentType::Rpgmv => &RPGMV,
        magika::ContentType::Rpm => &RPM,
        magika::ContentType::Rpyc => &RPYC,
        magika::ContentType::Rst => &RST,
        magika::ContentType::Rtf => &RTF,
        magika::ContentType::Rtm => &RTM,
        magika::ContentType::RtstrikeFont => &RTSTRIKE_FONT,
        magika::ContentType::Ruby => &RUBY,
        magika::ContentType::RubyRdocRi => &RUBY_RDOC_RI,
        magika::ContentType::RubyYarb => &RUBY_YARB,
        magika::ContentType::RupNinja2Patch => &RUP_NINJA2_PATCH,
        magika::ContentType::Rust => &RUST,
        magika::ContentType::Rzip => &RZIP,
        magika::ContentType::S3m => &S3M,
        magika::ContentType::S98 => &S98,
        magika::ContentType::Sabdu => &SABDU,
        magika::ContentType::SafariBinarycookies => &SAFARI_BINARYCOOKIES,
        magika::ContentType::Safetensors => &SAFETENSORS,
        magika::ContentType::SakuraWarsSbx => &SAKURA_WARS_SBX,
        magika::ContentType::SamCoupeCop => &SAM_COUPE_COP,
        magika::ContentType::SamsungPit => &SAMSUNG_PIT,
        magika::ContentType::SankomOzd => &SANKOM_OZD,
        magika::ContentType::SarCar => &SAR_CAR,
        magika::ContentType::Sarc => &SARC,
        magika::ContentType::Sas => &SAS,
        magika::ContentType::SawgrassDm => &SAWGRASS_DM,
        magika::ContentType::SblightBox => &SBLIGHT_BOX,
        magika::ContentType::SbstudioPac => &SBSTUDIO_PAC,
        magika::ContentType::Sc68Music => &SC68_MUSIC,
        magika::ContentType::ScRep => &SC_REP,
        magika::ContentType::Scala => &SCALA,
        magika::ContentType::ScaleformGfx => &SCALEFORM_GFX,
        magika::ContentType::SccMusixx => &SCC_MUSIXX,
        magika::ContentType::SchindlerSzip => &SCHINDLER_SZIP,
        magika::ContentType::Scp => &SCP,
        magika::ContentType::Scpt => &SCPT,
        magika::ContentType::ScratchSb => &SCRATCH_SB,
        magika::ContentType::Screencam => &SCREENCAM,
        magika::ContentType::Scriptwsf => &SCRIPTWSF,
        magika::ContentType::ScsHashfs => &SCS_HASHFS,
        magika::ContentType::ScsPmd => &SCS_PMD,
        magika::ContentType::ScsPmg => &SCS_PMG,
        magika::ContentType::ScsSiiEnc => &SCS_SII_ENC,
        magika::ContentType::ScsTobj => &SCS_TOBJ,
        magika::ContentType::Scss => &SCSS,
        magika::ContentType::ScummAmiga => &SCUMM_AMIGA,
        magika::ContentType::ScummCutsceneCut => &SCUMM_CUTSCENE_CUT,
        magika::ContentType::ScummRoomLfl => &SCUMM_ROOM_LFL,
        magika::ContentType::Scw => &SCW,
        magika::ContentType::SdcGdb => &SDC_GDB,
        magika::ContentType::Sds => &SDS,
        magika::ContentType::Sdw2Snp => &SDW2_SNP,
        magika::ContentType::SeamaFw => &SEAMA_FW,
        magika::ContentType::SeanConranScr => &SEAN_CONRAN_SCR,
        magika::ContentType::SecurityCfgCch => &SECURITY_CFG_CCH,
        magika::ContentType::SegaAfs => &SEGA_AFS,
        magika::ContentType::SegaFilm => &SEGA_FILM,
        magika::ContentType::SegaGsgm => &SEGA_GSGM,
        magika::ContentType::Segy => &SEGY,
        magika::ContentType::SelinuxPolicy => &SELINUX_POLICY,
        magika::ContentType::SelinuxPp => &SELINUX_PP,
        magika::ContentType::SemiOasis => &SEMI_OASIS,
        magika::ContentType::SenjutsuS7 => &SENJUTSU_S7,
        magika::ContentType::SercommFw => &SERCOMM_FW,
        magika::ContentType::SerifPageplusPublicationPpp => &SERIF_PAGEPLUS_PUBLICATION_PPP,
        magika::ContentType::SerpentStone => &SERPENT_STONE,
        magika::ContentType::Ses => &SES,
        magika::ContentType::Sevenzip => &SEVENZIP,
        magika::ContentType::Sff => &SFF,
        magika::ContentType::Sgc => &SGC,
        magika::ContentType::SgiMovie => &SGI_MOVIE,
        magika::ContentType::SgiRgb => &SGI_RGB,
        magika::ContentType::Sgml => &SGML,
        magika::ContentType::Sh3Smart => &SH3_SMART,
        magika::ContentType::Sh3d => &SH3D,
        magika::ContentType::Shapefile => &SHAPEFILE,
        magika::ContentType::Shell => &SHELL,
        magika::ContentType::ShinmeraSf3 => &SHINMERA_SF3,
        magika::ContentType::Shk => &SHK,
        magika::ContentType::Shn => &SHN,
        magika::ContentType::Sho => &SHO,
        magika::ContentType::ShockwaveW3d => &SHOCKWAVE_W3D,
        magika::ContentType::ShowpartnerHlp => &SHOWPARTNER_HLP,
        magika::ContentType::Sid => &SID,
        magika::ContentType::Sidmon2 => &SIDMON2,
        magika::ContentType::SierraAgiVol => &SIERRA_AGI_VOL,
        magika::ContentType::SierraRbt => &SIERRA_RBT,
        magika::ContentType::SierraSci32Pic => &SIERRA_SCI32_PIC,
        magika::ContentType::SierraSci32View => &SIERRA_SCI32_VIEW,
        magika::ContentType::SierraSciPatch => &SIERRA_SCI_PATCH,
        magika::ContentType::SierraSol => &SIERRA_SOL,
        magika::ContentType::SigmaX3f => &SIGMA_X3F,
        magika::ContentType::SignumFont => &SIGNUM_FONT,
        magika::ContentType::SignumSdo => &SIGNUM_SDO,
        magika::ContentType::SilabsGbl => &SILABS_GBL,
        magika::ContentType::SimhTap => &SIMH_TAP,
        magika::ContentType::Sims1Iff => &SIMS1_IFF,
        magika::ContentType::Sims4Thumbnail => &SIMS4_THUMBNAIL,
        magika::ContentType::SinclairQdos => &SINCLAIR_QDOS,
        magika::ContentType::SinclairQlF3 => &SINCLAIR_QL_F3,
        magika::ContentType::SingboxSrs => &SINGBOX_SRS,
        magika::ContentType::Sisx => &SISX,
        magika::ContentType::Sketchup => &SKETCHUP,
        magika::ContentType::Skops => &SKOPS,
        magika::ContentType::Sks => &SKS,
        magika::ContentType::SlimArchive => &SLIM_ARCHIVE,
        magika::ContentType::Smacker => &SMACKER,
        magika::ContentType::Smali => &SMALI,
        magika::ContentType::SmoothmovePan => &SMOOTHMOVE_PAN,
        magika::ContentType::SmsRom => &SMS_ROM,
        magika::ContentType::Sn2 => &SN2,
        magika::ContentType::Snap => &SNAP,
        magika::ContentType::SneventrackerSnm => &SNEVENTRACKER_SNM,
        magika::ContentType::SngplayObsm => &SNGPLAY_OBSM,
        magika::ContentType::Snoop => &SNOOP,
        magika::ContentType::SofFirmware => &SOF_FIRMWARE,
        magika::ContentType::SoftlibSlb => &SOFTLIB_SLB,
        magika::ContentType::SoftmakerDwr => &SOFTMAKER_DWR,
        magika::ContentType::SoftplanSpv => &SOFTPLAN_SPV,
        magika::ContentType::SolarisPkg => &SOLARIS_PKG,
        magika::ContentType::Solidity => &SOLIDITY,
        magika::ContentType::Sonarc => &SONARC,
        magika::ContentType::SonixInstr => &SONIX_INSTR,
        magika::ContentType::SonyOma => &SONY_OMA,
        magika::ContentType::SonyPicturegearPts => &SONY_PICTUREGEAR_PTS,
        magika::ContentType::SonyPmp => &SONY_PMP,
        magika::ContentType::SonyPspGim => &SONY_PSP_GIM,
        magika::ContentType::SonyPspGmo => &SONY_PSP_GMO,
        magika::ContentType::SonySgxd => &SONY_SGXD,
        magika::ContentType::SophosIde => &SOPHOS_IDE,
        magika::ContentType::SoprolSpl => &SOPROL_SPL,
        magika::ContentType::SoundControlSc => &SOUND_CONTROL_SC,
        magika::ContentType::SoundImagesSig => &SOUND_IMAGES_SIG,
        magika::ContentType::SoundfactoryPsf => &SOUNDFACTORY_PSF,
        magika::ContentType::SoundplayerSjs => &SOUNDPLAYER_SJS,
        magika::ContentType::Soundsmith => &SOUNDSMITH,
        magika::ContentType::Soundtracker26 => &SOUNDTRACKER_26,
        magika::ContentType::SoxAudio => &SOX_AUDIO,
        magika::ContentType::SpaceRangersGi => &SPACE_RANGERS_GI,
        magika::ContentType::Spc700 => &SPC700,
        magika::ContentType::Spd => &SPD,
        magika::ContentType::Spirv => &SPIRV,
        magika::ContentType::SpisLh5 => &SPIS_LH5,
        magika::ContentType::Spm => &SPM,
        magika::ContentType::SpreadtrumDhtbImg => &SPREADTRUM_DHTB_IMG,
        magika::ContentType::SpreadtrumPac => &SPREADTRUM_PAC,
        magika::ContentType::SpritepadSpd => &SPRITEPAD_SPD,
        magika::ContentType::Spss => &SPSS,
        magika::ContentType::Spu => &SPU,
        magika::ContentType::SqexAkb => &SQEX_AKB,
        magika::ContentType::Sql => &SQL,
        magika::ContentType::Sqlite => &SQLITE,
        magika::ContentType::Sqlite2 => &SQLITE2,
        magika::ContentType::SqliteJournal => &SQLITE_JOURNAL,
        magika::ContentType::SqliteWal => &SQLITE_WAL,
        magika::ContentType::SquareEnixScd => &SQUARE_ENIX_SCD,
        magika::ContentType::SquareEnixSpw => &SQUARE_ENIX_SPW,
        magika::ContentType::Squashfs => &SQUASHFS,
        magika::ContentType::SquirrelCnut => &SQUIRREL_CNUT,
        magika::ContentType::SqzArchive => &SQZ_ARCHIVE,
        magika::ContentType::Sra => &SRA,
        magika::ContentType::Srecord => &SRECORD,
        magika::ContentType::Srt => &SRT,
        magika::ContentType::SsbuParacobnPrc => &SSBU_PARACOBN_PRC,
        magika::ContentType::SsiGff => &SSI_GFF,
        magika::ContentType::StDfuse => &ST_DFUSE,
        magika::ContentType::Stacker => &STACKER,
        magika::ContentType::StadPac => &STAD_PAC,
        magika::ContentType::StalkerXrayOgf => &STALKER_XRAY_OGF,
        magika::ContentType::StalkerXrayThm => &STALKER_XRAY_THM,
        magika::ContentType::StarboundPak => &STARBOUND_PAK,
        magika::ContentType::StartrekkerNt => &STARTREKKER_NT,
        magika::ContentType::StarwriterLegacyVor => &STARWRITER_LEGACY_VOR,
        magika::ContentType::Stata => &STATA,
        magika::ContentType::StatisticaScr => &STATISTICA_SCR,
        magika::ContentType::StatisticaSta => &STATISTICA_STA,
        magika::ContentType::StatisticaStg => &STATISTICA_STG,
        magika::ContentType::Stc => &STC,
        magika::ContentType::SteelmcStlrSrg => &STEELMC_STLR_SRG,
        magika::ContentType::Step => &STEP,
        magika::ContentType::Stlbinary => &STLBINARY,
        magika::ContentType::Stltext => &STLTEXT,
        magika::ContentType::StockfishNnue => &STOCKFISH_NNUE,
        magika::ContentType::StonetrackerSps => &STONETRACKER_SPS,
        magika::ContentType::StormEngineTx => &STORM_ENGINE_TX,
        magika::ContentType::StormGm => &STORM_GM,
        magika::ContentType::StosAmos => &STOS_AMOS,
        magika::ContentType::StudentWritingCenter => &STUDENT_WRITING_CENTER,
        magika::ContentType::Sum => &SUM,
        magika::ContentType::SunXrs => &SUN_XRS,
        magika::ContentType::SunosToc => &SUNOS_TOC,
        magika::ContentType::Sunras => &SUNRAS,
        magika::ContentType::Sunvox => &SUNVOX,
        magika::ContentType::SunxiPackage => &SUNXI_PACKAGE,
        magika::ContentType::SuperscapeVirtualRealityFormatSvr => {
            &SUPERSCAPE_VIRTUAL_REALITY_FORMAT_SVR
        }
        magika::ContentType::Svd => &SVD,
        magika::ContentType::Svg => &SVG,
        magika::ContentType::Svm => &SVM,
        magika::ContentType::Swf => &SWF,
        magika::ContentType::Swift => &SWIFT,
        magika::ContentType::SwitchPfs0 => &SWITCH_PFS0,
        magika::ContentType::SymbolicsCarry => &SYMBOLICS_CARRY,
        magika::ContentType::Symmod => &SYMMOD,
        magika::ContentType::SynderSng => &SYNDER_SNG,
        magika::ContentType::SynopseSynlz => &SYNOPSE_SYNLZ,
        magika::ContentType::SynthDreamSdr => &SYNTH_DREAM_SDR,
        magika::ContentType::SystemdCatalog => &SYSTEMD_CATALOG,
        magika::ContentType::SystemdJournal => &SYSTEMD_JOURNAL,
        magika::ContentType::T98nextNfd => &T98NEXT_NFD,
        magika::ContentType::Tads3Image => &TADS3_IMAGE,
        magika::ContentType::TagAdventure => &TAG_ADVENTURE,
        magika::ContentType::TallyTcp => &TALLY_TCP,
        magika::ContentType::TangoproLib => &TANGOPRO_LIB,
        magika::ContentType::Tar => &TAR,
        magika::ContentType::TaxprepTfd => &TAXPREP_TFD,
        magika::ContentType::TbTbc => &TB_TBC,
        magika::ContentType::TbTbh => &TB_TBH,
        magika::ContentType::Tcl => &TCL,
        magika::ContentType::Td0 => &TD0,
        magika::ContentType::TdbdDatabase => &TDBD_DATABASE,
        magika::ContentType::Tds => &TDS,
        magika::ContentType::TekWfm => &TEK_WFM,
        magika::ContentType::TelegramTdf => &TELEGRAM_TDF,
        magika::ContentType::TelixSaltSlc => &TELIX_SALT_SLC,
        magika::ContentType::TencentTap => &TENCENT_TAP,
        magika::ContentType::Tensorflowjs => &TENSORFLOWJS,
        magika::ContentType::TerrariaRelogic => &TERRARIA_RELOGIC,
        magika::ContentType::TerrariaTmod => &TERRARIA_TMOD,
        magika::ContentType::Textproto => &TEXTPROTO,
        magika::ContentType::Tfe => &TFE,
        magika::ContentType::Tflite => &TFLITE,
        magika::ContentType::Tfm => &TFM,
        magika::ContentType::TfmxMod => &TFMX_MOD,
        magika::ContentType::Tfrecord => &TFRECORD,
        magika::ContentType::Tga => &TGA,
        magika::ContentType::Tgcf => &TGCF,
        magika::ContentType::ThebatSrb => &THEBAT_SRB,
        magika::ContentType::ThedrawTdf => &THEDRAW_TDF,
        magika::ContentType::Thilmera7ZpacT7p => &THILMERA7_ZPAC_T7P,
        magika::ContentType::Thumbsdb => &THUMBSDB,
        magika::ContentType::TiAis => &TI_AIS,
        magika::ContentType::TiBtsScript => &TI_BTS_SCRIPT,
        magika::ContentType::TiMulticore => &TI_MULTICORE,
        magika::ContentType::TiOad => &TI_OAD,
        magika::ContentType::TiRprc => &TI_RPRC,
        magika::ContentType::TiSmartampFw => &TI_SMARTAMP_FW,
        magika::ContentType::TiTpsUpg => &TI_TPS_UPG,
        magika::ContentType::Tiff => &TIFF,
        magika::ContentType::TimeOfWarD3e => &TIME_OF_WAR_D3E,
        magika::ContentType::TlsSessionNet443 => &TLS_SESSION_NET443,
        magika::ContentType::Tmdx => &TMDX,
        magika::ContentType::Tnef => &TNEF,
        magika::ContentType::Toml => &TOML,
        magika::ContentType::TomofonTomodel => &TOMOFON_TOMODEL,
        magika::ContentType::TomofonTopatch => &TOMOFON_TOPATCH,
        magika::ContentType::TomyTrackerSg => &TOMY_TRACKER_SG,
        magika::ContentType::Tops10Backup => &TOPS10_BACKUP,
        magika::ContentType::Tops20Dumper => &TOPS20_DUMPER,
        magika::ContentType::TopsolidRcf => &TOPSOLID_RCF,
        magika::ContentType::TopspeedTps => &TOPSPEED_TPS,
        magika::ContentType::Torchscript => &TORCHSCRIPT,
        magika::ContentType::TorquescriptDso => &TORQUESCRIPT_DSO,
        magika::ContentType::Torrent => &TORRENT,
        magika::ContentType::TotalWarPfh => &TOTAL_WAR_PFH,
        magika::ContentType::ToyotaCuw => &TOYOTA_CUW,
        magika::ContentType::TpOvr => &TP_OVR,
        magika::ContentType::TplinkFw => &TPLINK_FW,
        magika::ContentType::TplinkMinifs => &TPLINK_MINIFS,
        magika::ContentType::TrackerpackerMexx => &TRACKERPACKER_MEXX,
        magika::ContentType::TrackmaniaGbx => &TRACKMANIA_GBX,
        magika::ContentType::TrainzCdp => &TRAINZ_CDP,
        magika::ContentType::TrainzIm => &TRAINZ_IM,
        magika::ContentType::TresoritProfileBackup => &TRESORIT_PROFILE_BACKUP,
        magika::ContentType::TrocFs => &TROC_FS,
        magika::ContentType::TrphXpd => &TRPH_XPD,
        magika::ContentType::TrueColourPictureSpookyTrp => &TRUE_COLOUR_PICTURE_SPOOKY_TRP,
        magika::ContentType::Tsoundsystem => &TSOUNDSYSTEM,
        magika::ContentType::Tsv => &TSV,
        magika::ContentType::TtcXcfg => &TTC_XCFG,
        magika::ContentType::Ttf => &TTF,
        magika::ContentType::TundraTnd => &TUNDRA_TND,
        magika::ContentType::Tunefish2Tf2m => &TUNEFISH2_TF2M,
        magika::ContentType::TuneupTun => &TUNEUP_TUN,
        magika::ContentType::TurbopowerPophelp => &TURBOPOWER_POPHELP,
        magika::ContentType::TurbopowerTphelp => &TURBOPOWER_TPHELP,
        magika::ContentType::TurbopowerTphelp2 => &TURBOPOWER_TPHELP2,
        magika::ContentType::TurvisionRwp => &TURVISION_RWP,
        magika::ContentType::TwfbTsf => &TWFB_TSF,
        magika::ContentType::Twig => &TWIG,
        magika::ContentType::TwinTrackDmo => &TWIN_TRACK_DMO,
        magika::ContentType::TwinvqVqf => &TWINVQ_VQF,
        magika::ContentType::Txf => &TXF,
        magika::ContentType::Txt => &TXT,
        magika::ContentType::Typescript => &TYPESCRIPT,
        magika::ContentType::Tzif => &TZIF,
        magika::ContentType::Tzip => &TZIP,
        magika::ContentType::UaResume1 => &UA_RESUME1,
        magika::ContentType::UbfFont => &UBF_FONT,
        magika::ContentType::UbicomFw => &UBICOM_FW,
        magika::ContentType::Ubifs => &UBIFS,
        magika::ContentType::UbiquitiUbnt => &UBIQUITI_UBNT,
        magika::ContentType::UbisoftAnvilAcv => &UBISOFT_ANVIL_ACV,
        magika::ContentType::UbisoftJadeBf => &UBISOFT_JADE_BF,
        magika::ContentType::UbisoftXbt => &UBISOFT_XBT,
        magika::ContentType::Uboot => &UBOOT,
        magika::ContentType::UclinuxBflt => &UCLINUX_BFLT,
        magika::ContentType::UcsdCode => &UCSD_CODE,
        magika::ContentType::UcsdVol => &UCSD_VOL,
        magika::ContentType::Udf => &UDF,
        magika::ContentType::UefiCapsule => &UEFI_CAPSULE,
        magika::ContentType::UefiFv => &UEFI_FV,
        magika::ContentType::UefiTe => &UEFI_TE,
        magika::ContentType::Uf2 => &UF2,
        magika::ContentType::Uharc => &UHARC,
        magika::ContentType::UjamBeatmakerBlob => &UJAM_BEATMAKER_BLOB,
        magika::ContentType::Ult => &ULT,
        magika::ContentType::Ultima6M => &ULTIMA6_M,
        magika::ContentType::Undefined => &UNDEFINED,
        magika::ContentType::UniMikmod => &UNI_MIKMOD,
        magika::ContentType::UniflexImg => &UNIFLEX_IMG,
        magika::ContentType::UniqueDevUds => &UNIQUE_DEV_UDS,
        magika::ContentType::UnityAddressablesCatalog => &UNITY_ADDRESSABLES_CATALOG,
        magika::ContentType::UnityAssets => &UNITY_ASSETS,
        magika::ContentType::UnityEnlightenCaw => &UNITY_ENLIGHTEN_CAW,
        magika::ContentType::UnityEnlightenVis => &UNITY_ENLIGHTEN_VIS,
        magika::ContentType::UnityPreviewInfo => &UNITY_PREVIEW_INFO,
        magika::ContentType::UnityWebdata => &UNITY_WEBDATA,
        magika::ContentType::Unixcompress => &UNIXCOMPRESS,
        magika::ContentType::UnixwareBfs => &UNIXWARE_BFS,
        magika::ContentType::Unknown => &UNKNOWN,
        magika::ContentType::UnrealGvas => &UNREAL_GVAS,
        magika::ContentType::UnrealLocmeta => &UNREAL_LOCMETA,
        magika::ContentType::UnrealLocres => &UNREAL_LOCRES,
        magika::ContentType::UnrealManifest => &UNREAL_MANIFEST,
        magika::ContentType::UnrealPkg => &UNREAL_PKG,
        magika::ContentType::UnrealPsoCache => &UNREAL_PSO_CACHE,
        magika::ContentType::UnrealUsmap => &UNREAL_USMAP,
        magika::ContentType::UnrealUtoc => &UNREAL_UTOC,
        magika::ContentType::UnrealUtrace => &UNREAL_UTRACE,
        magika::ContentType::UnrealUz2 => &UNREAL_UZ2,
        magika::ContentType::Usd => &USD,
        magika::ContentType::UtauFrq => &UTAU_FRQ,
        magika::ContentType::V129Rdt => &V129_RDT,
        magika::ContentType::V8Bytecode => &V8_BYTECODE,
        magika::ContentType::Vag => &VAG,
        magika::ContentType::ValveAni => &VALVE_ANI,
        magika::ContentType::ValveHammerRmf => &VALVE_HAMMER_RMF,
        magika::ContentType::ValveMdl => &VALVE_MDL,
        magika::ContentType::ValveShaderVcs => &VALVE_SHADER_VCS,
        magika::ContentType::ValveToolsAssetInfo => &VALVE_TOOLS_ASSET_INFO,
        magika::ContentType::VamVab => &VAM_VAB,
        magika::ContentType::Vba => &VBA,
        magika::ContentType::Vbe => &VBE,
        magika::ContentType::Vcard => &VCARD,
        magika::ContentType::VcfGenomics => &VCF_GENOMICS,
        magika::ContentType::Vcs => &VCS,
        magika::ContentType::VctrVector => &VCTR_VECTOR,
        magika::ContentType::Vcxproj => &VCXPROJ,
        magika::ContentType::Vdex => &VDEX,
        magika::ContentType::VectorBlf => &VECTOR_BLF,
        magika::ContentType::VectorFwk => &VECTOR_FWK,
        magika::ContentType::Verilog => &VERILOG,
        magika::ContentType::Vfb => &VFB,
        magika::ContentType::VfdFdd => &VFD_FDD,
        magika::ContentType::Vgm => &VGM,
        magika::ContentType::Vhd => &VHD,
        magika::ContentType::Vhdl => &VHDL,
        magika::ContentType::Vhdx => &VHDX,
        magika::ContentType::Vib => &VIB,
        magika::ContentType::Vic20Rom => &VIC20_ROM,
        magika::ContentType::Vic2Huffman => &VIC2_HUFFMAN,
        magika::ContentType::VimSpellSpl => &VIM_SPELL_SPL,
        magika::ContentType::VimSwap => &VIM_SWAP,
        magika::ContentType::Vio2Score => &VIO2_SCORE,
        magika::ContentType::VipsImageV => &VIPS_IMAGE_V,
        magika::ContentType::VirtualdjVdjsample => &VIRTUALDJ_VDJSAMPLE,
        magika::ContentType::Visio => &VISIO,
        magika::ContentType::Vmdk => &VMDK,
        magika::ContentType::VmsExe => &VMS_EXE,
        magika::ContentType::VmsLib => &VMS_LIB,
        magika::ContentType::VmwareNvram => &VMWARE_NVRAM,
        magika::ContentType::VoodooVss => &VOODOO_VSS,
        magika::ContentType::VoxlapKv6 => &VOXLAP_KV6,
        magika::ContentType::VoyetraPatchBank => &VOYETRA_PATCH_BANK,
        magika::ContentType::Vpk => &VPK,
        magika::ContentType::Vst2Fxp => &VST2_FXP,
        magika::ContentType::Vst3Vstpreset => &VST3_VSTPRESET,
        magika::ContentType::VtechVrpLog => &VTECH_VRP_LOG,
        magika::ContentType::Vtf => &VTF,
        magika::ContentType::Vtk => &VTK,
        magika::ContentType::Vtt => &VTT,
        magika::ContentType::Vue => &VUE,
        magika::ContentType::Vvd => &VVD,
        magika::ContentType::VwSgo => &VW_SGO,
        magika::ContentType::W64 => &W64,
        magika::ContentType::Wad => &WAD,
        magika::ContentType::WamrAot => &WAMR_AOT,
        magika::ContentType::Warcraft3Map => &WARCRAFT3_MAP,
        magika::ContentType::Warcraft3Mdx => &WARCRAFT3_MDX,
        magika::ContentType::Warcraft3Replay => &WARCRAFT3_REPLAY,
        magika::ContentType::WarframeToc => &WARFRAME_TOC,
        magika::ContentType::Wasm => &WASM,
        magika::ContentType::WatcomWpk => &WATCOM_WPK,
        magika::ContentType::Wav => &WAV,
        magika::ContentType::Wbn => &WBN,
        magika::ContentType::Wcx => &WCX,
        magika::ContentType::Webm => &WEBM,
        magika::ContentType::Webp => &WEBP,
        magika::ContentType::WeltenbauerWbp => &WELTENBAUER_WBP,
        magika::ContentType::WestwoodCsf => &WESTWOOD_CSF,
        magika::ContentType::WestwoodVxl => &WESTWOOD_VXL,
        magika::ContentType::WgtFont => &WGT_FONT,
        magika::ContentType::WhatsappCrypt14 => &WHATSAPP_CRYPT14,
        magika::ContentType::WhitewaterActorIma => &WHITEWATER_ACTOR_IMA,
        magika::ContentType::WiiGeckoGct => &WII_GECKO_GCT,
        magika::ContentType::Wim => &WIM,
        magika::ContentType::Win31RegDat => &WIN31_REG_DAT,
        magika::ContentType::WinCal => &WIN_CAL,
        magika::ContentType::WinGrp => &WIN_GRP,
        magika::ContentType::WinKdump => &WIN_KDUMP,
        magika::ContentType::WinRestoreRpLog => &WIN_RESTORE_RP_LOG,
        magika::ContentType::WinSuperfetch => &WIN_SUPERFETCH,
        magika::ContentType::WinTaskJob => &WIN_TASK_JOB,
        magika::ContentType::WinboxCdb => &WINBOX_CDB,
        magika::ContentType::WinceB000ff => &WINCE_B000FF,
        magika::ContentType::WinceMsce => &WINCE_MSCE,
        magika::ContentType::WindowsAddressBookWab => &WINDOWS_ADDRESS_BOOK_WAB,
        magika::ContentType::WindowsCachesDb => &WINDOWS_CACHES_DB,
        magika::ContentType::WindowsCdfMs => &WINDOWS_CDF_MS,
        magika::ContentType::WindowsCodepageNls => &WINDOWS_CODEPAGE_NLS,
        magika::ContentType::WindowsImeMb => &WINDOWS_IME_MB,
        magika::ContentType::WindowsInstallerScript => &WINDOWS_INSTALLER_SCRIPT,
        magika::ContentType::WindowsPnf => &WINDOWS_PNF,
        magika::ContentType::WindowsPri => &WINDOWS_PRI,
        magika::ContentType::WindowsSdi => &WINDOWS_SDI,
        magika::ContentType::WinkWnk => &WINK_WNK,
        magika::ContentType::WinkWtp => &WINK_WTP,
        magika::ContentType::WinolsKp => &WINOLS_KP,
        magika::ContentType::Winregistry => &WINREGISTRY,
        magika::ContentType::Wiredtiger => &WIREDTIGER,
        magika::ContentType::WirelessRegdb => &WIRELESS_REGDB,
        magika::ContentType::WiresyncMomv2Omo => &WIRESYNC_MOMV2_OMO,
        magika::ContentType::Witcher2Dzip => &WITCHER2_DZIP,
        magika::ContentType::Witcher3Bundle => &WITCHER3_BUNDLE,
        magika::ContentType::Witcher3Sav => &WITCHER3_SAV,
        magika::ContentType::Witcher3W3speech => &WITCHER3_W3SPEECH,
        magika::ContentType::Witcher3W3strings => &WITCHER3_W3STRINGS,
        magika::ContentType::WitcherCr2w => &WITCHER_CR2W,
        magika::ContentType::Wkt => &WKT,
        magika::ContentType::Wma => &WMA,
        magika::ContentType::Wmf => &WMF,
        magika::ContentType::Wmv => &WMV,
        magika::ContentType::Woff => &WOFF,
        magika::ContentType::Woff2 => &WOFF2,
        magika::ContentType::WolfrpgMapMps => &WOLFRPG_MAP_MPS,
        magika::ContentType::WoodworksCwb => &WOODWORKS_CWB,
        magika::ContentType::Wordstar2000 => &WORDSTAR_2000,
        magika::ContentType::WorldcraftCmdseq => &WORLDCRAFT_CMDSEQ,
        magika::ContentType::WowDbc => &WOW_DBC,
        magika::ContentType::WowM2 => &WOW_M2,
        magika::ContentType::Woz => &WOZ,
        magika::ContentType::WpsDos => &WPS_DOS,
        magika::ContentType::Wpsplus => &WPSPLUS,
        magika::ContentType::WriteonWof => &WRITEON_WOF,
        magika::ContentType::Wsq => &WSQ,
        magika::ContentType::Wtv => &WTV,
        magika::ContentType::WwiseBnk => &WWISE_BNK,
        magika::ContentType::X64dbgTrace => &X64DBG_TRACE,
        magika::ContentType::X68000Mdx => &X68000_MDX,
        magika::ContentType::XactXgs => &XACT_XGS,
        magika::ContentType::XactXsb => &XACT_XSB,
        magika::ContentType::XactXwb => &XACT_XWB,
        magika::ContentType::Xalz => &XALZ,
        magika::ContentType::Xar => &XAR,
        magika::ContentType::XarImageFormat => &XAR_IMAGE_FORMAT,
        magika::ContentType::Xbin => &XBIN,
        magika::ContentType::XboxBdesXzp => &XBOX_BDES_XZP,
        magika::ContentType::XboxSdns => &XBOX_SDNS,
        magika::ContentType::XboxXbe => &XBOX_XBE,
        magika::ContentType::XcarFnt => &XCAR_FNT,
        magika::ContentType::Xcf => &XCF,
        magika::ContentType::Xcoff => &XCOFF,
        magika::ContentType::XcrArchive => &XCR_ARCHIVE,
        magika::ContentType::Xfs => &XFS,
        magika::ContentType::XiaomiHdr => &XIAOMI_HDR,
        magika::ContentType::XilamDerf => &XILAM_DERF,
        magika::ContentType::XilinxBit => &XILINX_BIT,
        magika::ContentType::XilinxXclbin => &XILINX_XCLBIN,
        magika::ContentType::XlPaintMax => &XL_PAINT_MAX,
        magika::ContentType::XlPaintRaw => &XL_PAINT_RAW,
        magika::ContentType::Xls => &XLS,
        magika::ContentType::Xlsb => &XLSB,
        magika::ContentType::Xlsx => &XLSX,
        magika::ContentType::Xm => &XM,
        magika::ContentType::Xml => &XML,
        magika::ContentType::XnaCourseCrs => &XNA_COURSE_CRS,
        magika::ContentType::Xnb => &XNB,
        magika::ContentType::Xp3 => &XP3,
        magika::ContentType::Xpi => &XPI,
        magika::ContentType::Xpk => &XPK,
        magika::ContentType::XplaneAcf => &XPLANE_ACF,
        magika::ContentType::XplaneDsf => &XPLANE_DSF,
        magika::ContentType::XplappPackage => &XPLAPP_PACKAGE,
        magika::ContentType::Xsd => &XSD,
        magika::ContentType::XsmMusic => &XSM_MUSIC,
        magika::ContentType::XvThumbnailP7 => &XV_THUMBNAIL_P7,
        magika::ContentType::Xvag => &XVAG,
        magika::ContentType::Xz => &XZ,
        magika::ContentType::Yaml => &YAML,
        magika::ContentType::Yara => &YARA,
        magika::ContentType::Yay0 => &YAY0,
        magika::ContentType::Yaz0 => &YAZ0,
        magika::ContentType::YglfFile => &YGLF_FILE,
        magika::ContentType::Ymst => &YMST,
        magika::ContentType::YurisYpf => &YURIS_YPF,
        magika::ContentType::Yz1 => &YZ1,
        magika::ContentType::Zbr1bgjbPrn => &ZBR1BGJB_PRN,
        magika::ContentType::Zbrush => &ZBRUSH,
        magika::ContentType::ZchunkZck => &ZCHUNK_ZCK,
        magika::ContentType::ZebraTpd => &ZEBRA_TPD,
        magika::ContentType::ZedalLibraryLv => &ZEDAL_LIBRARY_LV,
        magika::ContentType::ZeroplusAls => &ZEROPLUS_ALS,
        magika::ContentType::Zig => &ZIG,
        magika::ContentType::ZigbeeOta => &ZIGBEE_OTA,
        magika::ContentType::Zip => &ZIP,
        magika::ContentType::ZisofsFile => &ZISOFS_FILE,
        magika::ContentType::Zlibstream => &ZLIBSTREAM,
        magika::ContentType::Zmf => &ZMF,
        magika::ContentType::Zoo => &ZOO,
        magika::ContentType::ZoombrowserExThumbnailCacheInfo => {
            &ZOOMBROWSER_EX_THUMBNAIL_CACHE_INFO
        }
        magika::ContentType::ZosAr => &ZOS_AR,
        magika::ContentType::Zp1Archive => &ZP1_ARCHIVE,
        magika::ContentType::Zpaq => &ZPAQ,
        magika::ContentType::Zst => &ZST,
        magika::ContentType::ZucchiniPatch => &ZUCCHINI_PATCH,
        magika::ContentType::ZxChi => &ZX_CHI,
        magika::ContentType::ZxFtc => &ZX_FTC,
        magika::ContentType::ZxFxm => &ZX_FXM,
        magika::ContentType::ZxGtr => &ZX_GTR,
        magika::ContentType::ZxSqt => &ZX_SQT,
        magika::ContentType::ZxSt13 => &ZX_ST13,
        magika::ContentType::ZxStp => &ZX_STP,
        magika::ContentType::ZxTzx => &ZX_TZX,
        magika::ContentType::ZxVtx => &ZX_VTX,
        magika::ContentType::ZyxelBix => &ZYXEL_BIX,
        _ => unreachable!(),
    }
}
