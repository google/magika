# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
import ctypes as c
import json
import sys
import time
P=c.c_void_p
Create=c.CFUNCTYPE(c.c_int,c.c_size_t,c.POINTER(P),P,c.c_size_t)
Destroy=c.CFUNCTYPE(None,P)
class Api(c.Structure):
    _fields_=[('version',c.c_uint32),('size',c.c_uint32),('backend',c.c_uint32),('features',c.c_size_t),('labels',c.c_size_t),('create',Create),('destroy',Destroy),('session',P),('destroy_session',P),('run',P)]
lib=c.CDLL(sys.argv[1]);lib.magika_runtime_v1.restype=c.POINTER(Api)
api=lib.magika_runtime_v1().contents
assert (api.version,api.size,api.backend,api.features,api.labels)==(1,c.sizeof(Api),1,2048,214)
handle=P();error=c.create_string_buffer(2048)
start=time.perf_counter_ns();status=api.create(1,c.byref(handle),error,len(error));elapsed=time.perf_counter_ns()-start
assert status==0,error.value
api.destroy(handle)
print(json.dumps({'shared_prepare_ns':elapsed}))
