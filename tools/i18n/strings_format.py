"""Парсер бинарного формата Bethesda .strings (не .dlstrings/.ilstrings —
у тех есть дополнительный uint32-префикс длины перед каждой строкой,
здесь не нужен). Формат подтверждён вручную на реальном
skyrim_chinese.strings 2026-08-10."""
import struct


def parse_strings(path: str) -> dict[int, str]:
    with open(path, "rb") as f:
        data = f.read()

    count, _data_size = struct.unpack_from("<II", data, 0)
    table_offset = 8
    strings_start = table_offset + count * 8

    result: dict[int, str] = {}
    for i in range(count):
        string_id, rel_offset = struct.unpack_from("<II", data, table_offset + i * 8)
        start = strings_start + rel_offset
        end = data.find(b"\x00", start)
        result[string_id] = data[start:end].decode("utf-8", errors="replace")
    return result
