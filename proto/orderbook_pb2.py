# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: orderbook.proto
# Protobuf Python Version: 5.28.1
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import runtime_version as _runtime_version
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    5,
    28,
    1,
    '',
    'orderbook.proto'
)
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x0forderbook.proto\x12\torderbook\"\x07\n\x05\x45mpty\"Y\n\x07Summary\x12\x0e\n\x06spread\x18\x01 \x01(\x01\x12\x1e\n\x04\x62ids\x18\x02 \x03(\x0b\x32\x10.orderbook.Level\x12\x1e\n\x04\x61sks\x18\x03 \x03(\x0b\x32\x10.orderbook.Level\"8\n\x05Level\x12\x10\n\x08\x65xchange\x18\x01 \x01(\t\x12\r\n\x05price\x18\x02 \x01(\x01\x12\x0e\n\x06\x61mount\x18\x03 \x01(\x01\x32L\n\x13OrderbookAggregator\x12\x35\n\x0b\x42ookSummary\x12\x10.orderbook.Empty\x1a\x12.orderbook.Summary0\x01\x62\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'orderbook_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_EMPTY']._serialized_start=30
  _globals['_EMPTY']._serialized_end=37
  _globals['_SUMMARY']._serialized_start=39
  _globals['_SUMMARY']._serialized_end=128
  _globals['_LEVEL']._serialized_start=130
  _globals['_LEVEL']._serialized_end=186
  _globals['_ORDERBOOKAGGREGATOR']._serialized_start=188
  _globals['_ORDERBOOKAGGREGATOR']._serialized_end=264
# @@protoc_insertion_point(module_scope)