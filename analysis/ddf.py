#!/usr/bin/env python3

import sys
from struct import unpack

BS = 512
ANCHOR_LEN = 512
PDR_LEN = 64
PDE_LEN = 64
VDR_LEN = 64
VDE_LEN = 64
VDCR_BASE_LEN = 512
ENDIAN = ">"  # may become <

def print_field(label, value):
    result = "%02Xh" % value[0]
    for byte in value[1:]:
        result += " %02Xh" % byte
    print(f"{label}: {result}")

def identity(value):
    return value

def byte(value):
    result, = unpack(f"{ENDIAN}B", value)
    return result

def word(value):
    result, = unpack(f"{ENDIAN}H", value)
    return result

def dword(value):
    result, = unpack(f"{ENDIAN}L", value)
    return result

def qword(value):
    result, = unpack(f"{ENDIAN}Q", value)
    return result

ANCHOR_FIELD_LEN = [4, 4, 24, 8, 4, 4, 1, 1, 1, 13, 32, 8, 8, 1, 3, 4, 8, 2, 2, 2, 2, 2, 4, 50, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 256]
ANCHOR_FIELDS = [
    (11, qword, "Primary_Header_LBA"),
    (12, qword, "Secondary_Header_LBA"),
    (17, word, "Max_PD_Entries"),
    (18, word, "Max_VD_Entries"),
    (19, word, "Max_Partitions"),
    (20, word, "Configuration_Record_Length"),
    (21, word, "Max_Primary_Element_Entries"),
    (24, dword, "Controller_Data_Section"),
    (25, dword, "Controller_Data_Section_Length"),
    (26, dword, "Physical_Disk_Records_Section"),
    (27, dword, "Physical_Disk_Records_Section_Length"),
    (28, dword, "Virtual_Disk_Records_Section"),
    (29, dword, "Virtual_Disk_Records_Section_Length"),
    (30, dword, "Configuration_Records_Section"),
    (31, dword, "Configuration_Records_Section_Length"),
    (32, dword, "Physical_Disk_Data_Section"),
    (33, dword, "Physical_Disk_Data_Section_Length"),
    (34, dword, "BBM_Log_Section"),
    (35, dword, "BBM_Log_Section_Length"),
    (36, dword, "Diagnostic_Space"),
    (37, dword, "Diagnostic_Space_Length"),
    (38, dword, "Vendor_Specific_Logs_Section"),
    (39, dword, "Vendor_Specific_Logs_Section_Length"),
]

PDR_FIELD_LEN = [4, 4, 2, 2, 52]
PDR_FIELDS = [
    (2, word, "Populated_PDEs"),
]

PDE_FIELD_LEN = [24, 4, 2, 2, 8, 18, 2, 4]
PDE_FIELDS = [
    (0, identity, "PD_GUID"),
    (1, dword, "PD_Reference"),
]

VDR_FIELD_LEN = [4, 4, 2, 2, 52]
VDR_FIELDS = [
    (2, word, "Populated_VDEs"),
]

VDE_FIELD_LEN = [24, 2, 2, 4, 1, 1, 1, 13, 16]
VDE_FIELDS = [
    (0, identity, "VD_GUID"),
    (1, word, "VD_Number"),
    (2, identity, "Reserved"),
    (3, dword, "VD_Type"),
    (4, byte, "VD_State"),
    (5, byte, "Init_State"),
    (6, byte, "Partially_Optimal_Drive_Failures_Remaining"),
    (7, identity, "Reserved"),
    (8, identity, "VD_Name"),
]

VDCR_FIELD_LEN = [4, 4, 24, 4, 4, 24, 2, 1, 1, 1, 1, 1, 1, 8, 8, 2, 1, 5, 32, 8, 1, 3, 1, 2, 1, 1, 47, 192, 32, 32, 16, 16, 32]
VDCR_FIELDS = [
    (0, dword, "Signature"), # 4
    (1, dword, "CRC"), # 4
    (2, identity, "VD_GUID"), # 24
    (3, dword, "Timestamp"), # 4
    (4, dword, "Sequence_Number"), # 4
    # (5, identity, "Reserved"), # 24
    (6, word, "Primary_Element_Count"), # 2
    (7, byte, "Strip_Size"), # 1
    (8, byte, "Primary_RAID_Level"), # 1
    (9, byte, "RAID_Level_Qualifier"), # 1
    (10, byte, "Secondary_Element_Count"), # 1
    (11, byte, "Secondary_Element_Seq"), # 1
    (12, byte, "Secondary_RAID_Level"), # 1
    (13, qword, "Block_Count"), # 8
    (14, qword, "VD_Size"), # 8
    (15, word, "Block_Size"), # 2
    # (16, identity, ""), # 1
    # (17, identity, "Reserved"), # 5
    (18, identity, "Associated_Spares"), # 32
    # (19, identity, ""), # 8
    (20, byte, "BG_Rate"), # 1
    # (21, identity, "Reserved"), # 3
    # (22, identity, ""), # 1
    # (23, identity, ""), # 2
    # (24, identity, "Reserved"), # 1
    # (25, identity, ""), # 1
    # (26, identity, "Reserved"), # 47
    # (27, identity, "Reserved"), # 192
    # (28, identity, "V0"), # 32
    # (29, identity, "V1"), # 32
    # (30, identity, "V2"), # 16
    # (31, identity, "V3"), # 16
    # (32, identity, "Vendor Specific Scratch Space"), # 32
]

with open(sys.argv[1], "rb") as f:
    f.seek(0, 2)
    len_bytes = f.tell()
    len_sectors = len_bytes // BS


    print(">>> ddf anchor header")
    ddf_anchor_start = len_bytes - ANCHOR_LEN
    f.seek(ddf_anchor_start)

    ddf_anchor_raw = []
    for field_size in ANCHOR_FIELD_LEN:
        ddf_anchor_raw.append(f.read(field_size))
    for i, value in enumerate(ddf_anchor_raw):
        print_field(str(i), value)

    signature = ddf_anchor_raw[0]
    if signature == b"\x11\xde\x11\xde":
        print("warning: little endian?")
        ENDIAN = "<"
    else:
        assert signature == b"\xde\x11\xde\x11", repr(signature)

    ddf_anchor = {}
    for field_index, parser, key in ANCHOR_FIELDS:
        ddf_anchor[key] = parser(ddf_anchor_raw[field_index])

    for key in [
        "Primary_Header_LBA",
        "Secondary_Header_LBA",
    ]:
        lba = ddf_anchor[key]
        print("%s lba %016Xh" % (key, lba))

    for key in [
        "Max_PD_Entries",
        "Max_VD_Entries",
    ]:
        count = ddf_anchor[key]
        print("%s count %04Xh" % (key, count))

    for key in [
        "Configuration_Record_Length",
    ]:
        blocks = ddf_anchor[key]
        print("%s blocks %04Xh" % (key, blocks))

    for key in [
        "Controller_Data_Section",
        "Physical_Disk_Records_Section",
        "Virtual_Disk_Records_Section",
        "Configuration_Records_Section",
        "Physical_Disk_Data_Section",
        "BBM_Log_Section",
        "Diagnostic_Space",
        "Vendor_Specific_Logs_Section",
    ]:
        offset = ddf_anchor[key]
        size = ddf_anchor[f"{key}_Length"]
        print("%s offset %08Xh size %08Xh" % (key, offset, size))


    print(">>> physical disk records section")
    pdr_section_lba = ddf_anchor["Primary_Header_LBA"] + ddf_anchor["Physical_Disk_Records_Section"]
    f.seek(pdr_section_lba * BS)

    ddf_pdr_raw = []
    for field_size in PDR_FIELD_LEN:
        ddf_pdr_raw.append(f.read(field_size))
    for i, value in enumerate(ddf_pdr_raw):
        print_field(str(i), value)

    signature = ddf_pdr_raw[0]
    assert dword(signature) == 0x22222222

    ddf_pdr = {}
    for field_index, parser, key in PDR_FIELDS:
        ddf_pdr[key] = parser(ddf_pdr_raw[field_index])

    pde_count = ddf_pdr["Populated_PDEs"]
    print("PopulatedPDEs count %04Xh" % pde_count)

    i = 0
    pde_seen = 0
    assert ddf_pdr["Populated_PDEs"] <= ddf_anchor["Max_PD_Entries"]
    while pde_seen < ddf_pdr["Populated_PDEs"]:
        f.seek(pdr_section_lba * BS + PDR_LEN + i * PDE_LEN)
        i += 1

        ddf_pde_raw = []
        for field_size in PDE_FIELD_LEN:
            ddf_pde_raw.append(f.read(field_size))
        # for i, value in enumerate(ddf_pde_raw):
        #     print_field(str(i), value)

        if ddf_pde_raw[0] == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF":
            continue

        ddf_pde = {}
        for field_index, parser, key in PDE_FIELDS:
            ddf_pde[key] = parser(ddf_pde_raw[field_index])

        print("pde PD_GUID %r PD_Reference %08Xh" % (ddf_pde["PD_GUID"], ddf_pde["PD_Reference"]))
        pde_seen += 1


    print(">>> virtual disk records section")
    vdr_section_lba = ddf_anchor["Primary_Header_LBA"] + ddf_anchor["Virtual_Disk_Records_Section"]
    f.seek(vdr_section_lba * BS)

    ddf_vdr_raw = []
    for field_size in VDR_FIELD_LEN:
        ddf_vdr_raw.append(f.read(field_size))
    for i, value in enumerate(ddf_vdr_raw):
        print_field(str(i), value)

    signature = ddf_vdr_raw[0]
    assert dword(signature) == 0xDDDDDDDD

    ddf_vdr = {}
    for field_index, parser, key in VDR_FIELDS:
        ddf_vdr[key] = parser(ddf_vdr_raw[field_index])

    vde_count = ddf_vdr["Populated_VDEs"]
    print("PopulatedVDEs count %04Xh" % vde_count)

    i = 0
    vde_seen = 0
    assert ddf_vdr["Populated_VDEs"] <= ddf_anchor["Max_VD_Entries"]
    while vde_seen < ddf_vdr["Populated_VDEs"]:
        f.seek(vdr_section_lba * BS + VDR_LEN + i * VDE_LEN)
        i += 1

        ddf_vde_raw = []
        for field_size in VDE_FIELD_LEN:
            ddf_vde_raw.append(f.read(field_size))
        # for i, value in enumerate(ddf_vde_raw):
        #     print_field(str(i), value)

        if ddf_vde_raw[0] == b"\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF":
            continue

        ddf_vde = {}
        for field_index, parser, key in VDE_FIELDS:
            ddf_vde[key] = parser(ddf_vde_raw[field_index])
        
        print("vde guid %r name %r" % (ddf_vde["VD_GUID"], ddf_vde["VD_Name"]))
        vde_seen += 1


    print(">>> configuration record section")
    cr_section_lba = ddf_anchor["Primary_Header_LBA"] + ddf_anchor["Configuration_Records_Section"]
    f.seek(cr_section_lba * BS)

    cr_section_blocks = ddf_anchor["Configuration_Records_Section_Length"]
    cr_record_blocks = ddf_anchor["Configuration_Record_Length"]
    cr_record_count = cr_section_blocks / cr_record_blocks
    assert int(cr_record_count) == ddf_anchor["Max_Partitions"] + 1
    print("%f records" % cr_record_count)

    for i in range(int(cr_record_count)):
        f.seek(cr_section_lba * BS + i * cr_record_blocks)

        ddf_vdcr_raw = []
        for field_size in VDCR_FIELD_LEN:
            ddf_vdcr_raw.append(f.read(field_size))
        # for i, value in enumerate(ddf_vdcr_raw):
        #     print_field(str(i), value)

        signature = ddf_vdcr_raw[0]
        print("vdcr signature %08Xh" % dword(signature))

        if dword(signature) != 0xEEEEEEEE:
            continue

        ddf_vdcr = {}
        for field_index, parser, key in VDCR_FIELDS:
            ddf_vdcr[key] = parser(ddf_vdcr_raw[field_index])

        for key in [
            "VD_GUID",
            "Primary_Element_Count",
            "Strip_Size",
        ]:
            print("     %s %r" % (key, ddf_vdcr[key]))

        for key in [
            "Primary_RAID_Level",
            "RAID_Level_Qualifier",
        ]:
            print("     %s %02Xh" % (key, ddf_vdcr[key]))

        print("     Physical_Disk_Sequence")
        f.seek(cr_section_lba * BS + i * cr_record_blocks + VDCR_BASE_LEN)

        j = 0
        assert ddf_vdcr["Primary_Element_Count"] <= ddf_anchor["Max_Primary_Element_Entries"]
        while j < ddf_vdcr["Primary_Element_Count"]:
            value = dword(f.read(4))
            if value == 0xFFFFFFFF:
                continue
            print("         PD_Reference %08Xh" % value)
            j += 1
