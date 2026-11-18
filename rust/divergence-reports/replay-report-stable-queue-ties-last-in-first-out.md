# App replay

Build: `/var/folders/2n/mgyt43vn1391pyy5q8wv31kh0000gn/T/pekko-mutants-py4t6s5v/target-stable-queue-ties-last-in-first-out/release/examples/classic-parity`

Passed: 1410/1417

Fallback rate: not reported by this build

## Failing cases

### ok-user-info-lone-surrog

Arguments: `path-from-string`

status:

```diff
expected exit code 0, got exit code 2
```

stdout:

```diff
- {"ok":true,"toString":"pekko://sys\ud800@host:2552/user","name":"user","elements":["user"],"uid":0,"toStringWithoutAddress":"/user","toSerializationFormat":"pekko://sys\ud800@host:2552/user","address":{"protocol":"pekko","system":"sys\ud800","host":"host","port":2552,"toString":"pekko://sys\ud800@host:2552"}}
```

stderr:

```diff
+ input outside the API domain: lone UTF-16 surrogate (public inputs are &str; decision Q-X-17)
```

### uc-0342

Arguments: `util-core-case`

stdout:

```diff
- SQ-01 FIFO among equal priorities -> ok t t t t t 5 a1 a1 a2 a3 b1 b2 null null 0
+ SQ-01 FIFO among equal priorities -> ok t t t t t 5 a3 a3 a2 a1 b2 b1 null null 0
```

### uc-0343

Arguments: `util-core-case`

stdout:

```diff
- SQ-02 lower priority first, FIFO for ties (blocking queue) -> ok t t t t t [y1,y2,z1,x1,x2] 0
+ SQ-02 lower priority first, FIFO for ties (blocking queue) -> ok t t t t t [y2,y1,z1,x2,x1] 0
```

### uc-0344

Arguments: `util-core-case`

stdout:

```diff
- SQ-03 many equal priorities keep the order of arrival -> ok 0 [] t t t t t t t t t t t t t [e0,e1,e2,e3,e4,e5,e6,e7,e8,e9,e10,e11,e12]
+ SQ-03 many equal priorities keep the order of arrival -> ok 0 [] t t t t t t t t t t t t t [e12,e11,e10,e9,e8,e7,e6,e5,e4,e3,e2,e1,e0]
```

### uc-0346

Arguments: `util-core-case`

stdout:

```diff
- SQ-92 random sequence seed 92 -> ok null t t t t t t e2 t t e5 t t e8 t t t t t e14 t t e16 t e3 e9 t t t t t t t t e19 t t t t t t t t t e30 t t e34 t t t e36 e23 e33 e11 t t t e42 t t t t e17 t e18 t e48 t e49 e20 t e50 t t e51 e21 e24 e26 t t t t t t e53 t e57 t t t t e63 e55 e58 e61 e28 t t t t e64 e31 t t t t e68 e69 e35 t t t t t e72 t t t e73 t t t e77 e82 e40 t t t e83 e84 t t t t t e86 e47 t e56 t t e59 e60 e66 t t t t e95 t t t t t t t e97 t t t e98 t t e99 t t e101 t t t e102 t t e106 t t e117 t t e96 t e103 t t t e123 e107 t e109 t e111 e112 t t t e127 e124 t e129 t t t t e134 t t t e135 t e138 t e70 t t t t t e142 e75 t t t e147 e76 e78 t e90 t e93 t t t t e152 t e154 t e104 t e156 e110 e115 t t e118 e120 t t t t t t t t t t t t t t t t t t t t t e163 t t t t t e166 t t t t e167 t e177 t t t e180 t t t t e191 e192 e194 e165 t t e173 e181 e183 t t e122 e132 e139 e145 t t e146 t e148 t t t e204 e205 t e207 e158 e169 t t e209 t t e170 t t e178 t t e179 t e216 t t e184 t t t t t e219 t e222 t e186 t e226 e187 t t t t e196 t t t e198 t t e234 t e236 e200 t e211 t t e239 t t t t t t e241 t t t t t e247 e250 t t t t t t e252 t t e242 t t t e246 e256 t t t e257 t e259 t e266 e260 t e267 t t t e265 t e270 e212 t e272 t t t t t e274 t t e275 t e278 e280 t t t t t t t t t t t e281 t t e284 t t e287 e293 e294 t t e297 t t t e296 e298 t t t t e301 e302 t t e215 t t t t t t t t t t t e313 e314 t t t t t e316 t e322 e307 e308 t t t e326 t t e323 t e328 t t t t t t t t t e332 t t t t t e342 e343 e329 t e333 t t t e338 e218 t t t t t e348 t e352 e223 e227 t t t e354 e228 e229 t t t e359 t e360 t e361 t e231 t t e364 t e365 e233 t t e235 e237 t t t t e368 t t t t t t t t t e372 t t e377 t e383 t t t t t t e387 t t t t t t t t e370 t t t e398 203 [e400,e378,e381,e382,e384,e386,e391,e397,e240,e243,e255,e263,e273,e279,e283,e286,e288,e292,e295,e304,e312,e315,e320,e327,e334,e335,e336,e346,e357,e358,e363,e371,e374,e379,e395,e396,e4,e6,e7,e10,e27,e37,e38,e39,e43,e46,e52,e62,e67,e71,e85,e89,e91,e94,e100,e105,e119,e130,e149,e157,e159,e164,e172,e175,e185,e188,e190,e195,e197,e201,e202,e203,e206,e208,e210,e213,e214,e217,e221,e224,e230,e232,e244,e245,e251,e258,e262,e264,e271,e277,e289,e290,e291,e300,e309,e317,e321,e324,e330,e331,e337,e339,e341,e344,e345,e347,e353,e356,e367,e369,e375,e376,e385,e389,e390,e392,e0,e1,e12,e13,e15,e22,e25,e29,e32,e41,e44,e45,e54,e65,e74,e79,e80,e81,e87,e88,e92,e108,e113,e114,e116,e121,e125,e126,e128,e131,e133,e136,e137,e140,e141,e143,e144,e150,e151,e153,e155,e160,e161,e162,e168,e171,e174,e176,e182,e189,e193,e199,e220,e225,e238,e248,e249,e253,e254,e261,e268,e269,e276,e282,e285,e299,e303,e305,e306,e310,e311,e318,e319,e325,e340,e349,e350,e351,e355,e362,e366,e373,e380,e388,e393,e394,e399]
+ SQ-92 random sequence seed 92 -> ok null t t t t t t e2 t t e5 t t e8 t t t t t e14 t t e16 t e17 e11 t t t t t t t t e23 t t t t t t t t t e34 t t e36 t t t e30 e33 e19 e35 t t t e42 t t t t e40 t e47 t e48 t e49 e31 t e50 t t e51 e28 e26 e24 t t t t t t e57 t e53 t t t t e63 e61 e58 e55 e60 t t t t e64 e66 t t t t e69 e68 e70 t t t t t e73 t t t e77 t t t e72 e82 e78 t t t e84 e83 t t t t t e86 e90 t e76 t t e93 e75 e59 t t t t e97 t t t t t t t e102 t t t e106 t t e101 t t e99 t t t e98 t t e95 t t e117 t t e112 t e111 t t t e123 e124 t e109 t e107 e103 t t t e127 e129 t e96 t t t t e134 t t t e135 t e138 t e139 t t t t t e142 e132 t t t e147 e146 e145 t e148 t e122 t t t t e152 t e154 t e120 t e156 e118 e115 t t e158 e110 t t t t t t t t t t t t t t t t t t t t t e177 t t t t t e180 t t t t e167 t e166 t t t e192 t t t t e194 e191 e163 e183 t t e181 e173 e165 t t e200 e198 e196 e187 t t e186 t e184 t t t e204 e205 t e207 e179 e178 t t e209 t t e211 t t e212 t t e215 t e216 t t e218 t t t t t e222 t e219 t e223 t e226 e170 t t t t e229 t t t e233 t t e234 t e236 e235 t e237 t t e239 t t t t t t e241 t t t t t e250 e247 t t t t t t e252 t t e257 t t t e260 e259 t t t e256 t e265 t e266 e246 t e267 t t t e270 t e242 e263 t e272 t t t t t e275 t t e278 t e274 e280 t t t t t t t t t t t e284 t t e281 t t e294 e293 e287 t t e297 t t t e298 e296 t t t t e302 e301 t t e304 t t t t t t t t t t t e316 e314 t t t t t e322 t e313 e323 e308 t t t e326 t t e328 t e329 t t t t t t t t t e332 t t t t t e343 e342 e338 t e333 t t t e307 e346 t t t t t e352 t e348 e336 e335 t t t e354 e334 e327 t t t e359 t e360 t e361 t e358 t t e364 t e365 e363 t t e357 e320 t t t t e370 t t t t t t t t t e377 t t e372 t e383 t t t t t t e387 t t t t t t t t e397 t t t e400 203 [e398,e391,e386,e384,e382,e381,e378,e368,e396,e395,e379,e374,e371,e315,e312,e295,e292,e288,e286,e283,e279,e273,e255,e243,e240,e231,e228,e227,e169,e104,e56,e21,e20,e18,e9,e3,e392,e390,e389,e385,e376,e375,e369,e367,e356,e353,e347,e345,e344,e341,e339,e337,e331,e330,e324,e321,e317,e309,e300,e291,e290,e289,e277,e271,e264,e262,e258,e251,e245,e244,e232,e230,e224,e221,e217,e214,e213,e210,e208,e206,e203,e202,e201,e197,e195,e190,e188,e185,e175,e172,e164,e159,e157,e149,e130,e119,e105,e100,e94,e91,e89,e85,e71,e67,e62,e52,e46,e43,e39,e38,e37,e27,e10,e7,e6,e4,e399,e394,e393,e388,e380,e373,e366,e362,e355,e351,e350,e349,e340,e325,e319,e318,e311,e310,e306,e305,e303,e299,e285,e282,e276,e269,e268,e261,e254,e253,e249,e248,e238,e225,e220,e199,e193,e189,e182,e176,e174,e171,e168,e162,e161,e160,e155,e153,e151,e150,e144,e143,e141,e140,e137,e136,e133,e131,e128,e126,e125,e121,e116,e114,e113,e108,e92,e88,e87,e81,e80,e79,e74,e65,e54,e45,e44,e41,e32,e29,e25,e22,e15,e13,e12,e1,e0]
```

### uc-0347

Arguments: `util-core-case`

stdout:

```diff
- SQ-93 random sequence seed 93 -> ok t t t t t e2 t t t e6 e7 t e1 t t t e9 e4 e8 e10 t t t t e12 t t e17 e15 e16 t t t e19 t t t t t e21 e23 t t t t t t t e24 t t e28 t t e36 t t e37 t t e39 e32 e25 t t e41 e42 t t t e43 t e44 e45 t t e27 e31 t e49 e34 t t t e52 t e53 e35 t e46 e47 t t e55 t e56 t t t e50 e54 t e58 t t t t t t e64 e67 e62 t t e68 t e66 e26 e38 t t t t t e73 t t e71 e75 t t e76 t e80 e48 t t t t t e81 t t t e85 t t t e86 t e88 t t t e89 e91 t t e95 t t t t t t e100 t t t t e83 e90 t e93 t t t e111 e96 t t t t t e112 e113 e115 t t e118 t e104 e106 e108 e110 e114 e102 e57 t t t t e122 t t e123 t t t t e129 e120 t e130 t e131 t t t e132 t e133 e135 e61 t e136 t e137 t e63 t t e140 e139 t t e141 t e142 t e144 e65 t t e145 t t t t t t t t t t t t t t t e150 t e152 t t t t t t e153 t t e155 t e156 t e159 e164 t t t t t e168 t e170 t t t t e180 t t t e183 e151 t t t t t t t e186 t t e190 e191 t e194 t t e195 t e165 e171 e174 t t e177 t t t t t t t t t e203 t t e205 e210 e187 e188 e200 e209 t e146 t e213 e147 e157 t e160 e161 e167 t t t t e218 t t t t e219 e222 e216 e172 t t t t e173 t e227 t t e228 t t t t e230 e231 t e234 e232 t e179 t t e237 t t t t t e239 e181 t t t t e244 e243 t t t e247 e249 e182 t t e250 t t t e253 e192 e197 t t e255 t e198 t e258 t t t t t t t e204 t e266 e214 t t e217 t e269 t e221 t t e272 e225 e229 t t t t t t e273 t t t t t t t t t t t t e274 t t t t t t t t e282 t e288 e293 t t t t t t t t e294 t t e279 t t t e311 t t t t e315 t e285 t t t t t e321 t t t t t t t t e328 t t e287 e292 t e333 e298 t t e301 e304 t e307 t t t e318 e319 t e320 t t t t e342 e330 t e338 e343 t t t e347 t e236 t t t e238 t t e240 t e355 e242 t e256 t e265 t t e271 t e275 t e277 t t t t t t t t t t e368 e362 t t e363 e364 t t t e371 e372 e373 t t t t t t t e378 t 170 [e383,e376,e381,e280,e290,e291,e297,e309,e310,e322,e331,e334,e337,e344,e345,e348,e350,e352,e354,e356,e359,e365,e366,e377,e384,e70,e74,e77,e78,e82,e87,e92,e98,e116,e121,e124,e126,e128,e134,e143,e154,e158,e163,e166,e176,e184,e189,e207,e220,e223,e224,e226,e251,e252,e257,e260,e262,e267,e268,e270,e281,e283,e286,e289,e295,e296,e300,e302,e312,e314,e316,e317,e323,e324,e325,e326,e327,e335,e340,e341,e349,e353,e357,e369,e374,e375,e379,e0,e3,e5,e11,e13,e14,e18,e20,e22,e29,e30,e33,e40,e51,e59,e60,e69,e72,e79,e84,e94,e97,e99,e101,e103,e105,e107,e109,e117,e119,e125,e127,e138,e148,e149,e162,e169,e175,e178,e185,e193,e196,e199,e201,e202,e206,e208,e211,e212,e215,e233,e235,e241,e245,e246,e248,e254,e259,e261,e263,e264,e276,e278,e284,e299,e303,e305,e306,e308,e313,e329,e332,e336,e339,e346,e351,e358,e360,e361,e367,e370,e380,e382]
+ SQ-93 random sequence seed 93 -> ok t t t t t e2 t t t e7 e6 t e8 t t t e9 e10 e4 e1 t t t t e15 t t e17 e16 e12 t t t e19 t t t t t e24 e23 t t t t t t t e32 t t e28 t t e36 t t e37 t t e39 e21 e35 t t e42 e41 t t t e45 t e44 e43 t t e47 e46 t e49 e34 t t t e52 t e53 e50 t e54 e31 t t e56 t e55 t t t e58 e27 t e25 t t t t t t e67 e64 e66 t t e68 t e62 e70 e65 t t t t t e73 t t e76 e75 t t e71 t e80 e78 t t t t t e85 t t t e88 t t t e91 t e89 t t t e95 e86 t t e81 t t t t t t e100 t t t t e106 e104 t e108 t t t e111 e110 t t t t t e115 e113 e112 t t e118 t e114 e96 e93 e90 e83 e102 e116 t t t t e123 t t e122 t t t t e129 e120 t e130 t e131 t t t e132 t e135 e133 e134 t e136 t e137 t e128 t t e140 e139 t t e142 t e141 t e144 e143 t t e145 t t t t t t t t t t t t t t t e159 t e156 t t t t t t e168 t t e170 t e164 t e155 e153 t t t t t e152 t e150 t t t t e180 t t t e183 e177 t t t t t t t e191 t t e194 e190 t e195 t t e186 t e188 e187 e174 t t e200 t t t t t t t t t e205 t t e210 e203 e209 e171 e165 e151 t e204 t e213 e198 e197 t e214 e192 e182 t t t t e218 t t t t e222 e219 e216 e221 t t t t e225 t e227 t t e228 t t t t e232 e231 t e234 e230 t e229 t t e237 t t t t t e239 e242 t t t t e244 e243 t t t e249 e247 e240 t t e250 t t t e253 e238 e236 t t e255 t e256 t e258 t t t t t t t e265 t e266 e217 t t e181 t e269 t e179 t t e272 e271 e173 t t t t t t e274 t t t t t t t t t t t t e288 t t t t t t t t e294 t e293 e282 t t t t t t t t e273 t t e307 t t t e311 t t t t e315 t e304 t t t t t e321 t t t t t t t t e328 t t e330 e320 t e333 e319 t t e318 e301 t e298 t t t e338 e292 t e287 t t t t e342 e343 t e285 e279 t t t e347 t e348 t t t e352 t t e354 t e355 e350 t e356 t e345 t t e359 t e344 t e337 t t t t t t t t t t e368 e371 t t e373 e372 t t t e376 e364 e363 t t t t t t t e383 t 170 [e378,e381,e362,e384,e377,e366,e365,e334,e331,e322,e310,e309,e297,e291,e290,e280,e277,e275,e172,e167,e161,e160,e157,e147,e146,e379,e375,e374,e369,e357,e353,e349,e341,e340,e335,e327,e326,e325,e324,e323,e317,e316,e314,e312,e302,e300,e296,e295,e289,e286,e283,e281,e270,e268,e267,e262,e260,e257,e252,e251,e226,e224,e223,e220,e207,e189,e184,e176,e166,e163,e158,e154,e126,e124,e121,e98,e92,e87,e82,e77,e74,e63,e61,e57,e48,e38,e26,e382,e380,e370,e367,e361,e360,e358,e351,e346,e339,e336,e332,e329,e313,e308,e306,e305,e303,e299,e284,e278,e276,e264,e263,e261,e259,e254,e248,e246,e245,e241,e235,e233,e215,e212,e211,e208,e206,e202,e201,e199,e196,e193,e185,e178,e175,e169,e162,e149,e148,e138,e127,e125,e119,e117,e109,e107,e105,e103,e101,e99,e97,e94,e84,e79,e72,e69,e60,e59,e51,e40,e33,e30,e29,e22,e20,e18,e14,e13,e11,e5,e3,e0]
```

### uc-0348

Arguments: `util-core-case`

stdout:

```diff
- SQ-94 random sequence on the blocking queue -> ok t t e1 t t t t t e0 t t t t e8 t e11 e2 t t e13 t t t t t t t t e14 t t e21 t e3 e4 t e6 t t e27 e9 t e15 e19 e25 e26 t t e5 t t e31 e10 e12 t t t t t e34 e16 t t e38 e39 t t t e40 e41 e42 e18 t t t t e22 t t e23 t t t t e52 e28 t t t e54 e53 t t e55 t e58 t t t t t t e59 e60 t t e64 t t t t e65 t t e66 t t t t t t t e70 e77 t t e78 t e80 e81 t t e84 t t t e82 e30 e36 t e88 e44 e48 t t t e90 e89 e50 t t t t e94 t t t t e97 t t e98 e101 t e51 t t e57 t t e105 t e61 t t t e108 t e110 e63 e73 e86 t t t e114 e87 e93 t e115 t t t t e103 e109 t t t t t e116 e119 t e125 t e126 e120 t t t t t t t e129 e127 t e134 t t e132 t t t t e139 t t e121 t t t t t t t t t t t t t e144 t e145 t t e151 e152 t t t t t t e155 e162 e147 t e149 t e166 e150 t t e159 t t t t t t t e171 t e172 t e170 t e176 t t e180 t e181 t e177 t t t t e178 e124 e130 e131 t e133 t e135 t t e190 t t e138 e140 t e143 t e194 t e157 t e196 t e161 t e198 t t e168 e175 e179 t t t t t e205 e203 t t e206 t t t t t t t e213 t t t t t e207 e209 e214 e215 t t t t t e220 t t t e226 t t t t t t t t t e232 e221 t e222 e224 e225 e227 t t t e239 t t t e228 t e244 t t t t t t t e231 e233 t e241 t t t t e255 e242 t t t t t t e247 t e254 e260 t t t t t e185 t e269 e199 e202 t t e271 t t e272 e273 e219 t e230 e248 t t e276 t t t t t t e278 t t t e282 e284 t t e277 e279 t t t t t e289 t t t t t t e293 t t e299 e300 t e280 t e287 t e291 t t t t t t t e310 t t e311 t t t t e315 t t t e316 t t t t t e318 t e319 t e320 e324 e294 t t e296 e313 t t e314 t e321 e329 e331 t t t t t t t t t t e333 e336 t t e339 t t t t t t e344 t t e350 t t e351 t t t t t t e353 t e355 t t t t t e358 e360 t t e361 e337 e338 t t e340 e342 t t e371 e343 e346 t e372 e348 e365 t e369 t t t e376 t t e378 t t t t e382 t t t t 174 [e384,e252,e256,e257,e258,e259,e265,e267,e268,e274,e275,e288,e295,e306,e307,e309,e322,e325,e335,e345,e347,e349,e356,e357,e364,e366,e368,e374,e377,e383,e17,e20,e24,e33,e37,e43,e45,e49,e69,e72,e74,e79,e83,e85,e92,e96,e100,e104,e106,e111,e118,e123,e137,e141,e142,e148,e154,e156,e160,e164,e167,e173,e183,e186,e187,e188,e191,e193,e195,e197,e201,e208,e211,e217,e218,e235,e236,e249,e251,e253,e261,e263,e264,e283,e292,e297,e298,e301,e302,e323,e326,e341,e354,e359,e367,e381,e385,e386,e7,e29,e32,e35,e46,e47,e56,e62,e67,e68,e71,e75,e76,e91,e95,e99,e102,e107,e112,e113,e117,e122,e128,e136,e146,e153,e158,e163,e165,e169,e174,e182,e184,e189,e192,e200,e204,e210,e212,e216,e223,e229,e234,e237,e238,e240,e243,e245,e246,e250,e262,e266,e270,e281,e285,e286,e290,e303,e304,e305,e308,e312,e317,e327,e328,e330,e332,e334,e352,e362,e363,e370,e373,e375,e379,e380]
+ SQ-94 random sequence on the blocking queue -> ok t t e1 t t t t t e6 t t t t e8 t e11 e9 t t e13 t t t t t t t t e21 t t e14 t e19 e15 t e25 t t e27 e26 t e4 e3 e2 e0 t t e30 t t e31 e28 e23 t t t t t e34 e36 t t e39 e38 t t t e42 e41 e40 e22 t t t t e44 t t e48 t t t t e52 e51 t t t e54 e55 t t e53 t e58 t t t t t t e64 e60 t t e66 t t t t e65 t t e59 t t t t t t t e78 e77 t t e81 t e82 e80 t t e84 t t t e70 e87 e86 t e88 e73 e63 t t t e90 e89 e61 t t t t e94 t t t t e97 t t e101 e98 t e93 t t e103 t t e105 t e57 t t t e110 t e108 e109 e50 e18 t t t e114 e16 e12 t e115 t t t t e119 e116 t t t t t e124 e121 t e125 t e126 e120 t t t t t t t e129 e132 t e134 t t e127 t t t t e139 t t e140 t t t t t t t t t t t t t e155 t e152 t t e151 e145 t t t t t t e162 e144 e159 t e150 t e166 e149 t t e147 t t t t t t t e172 t e171 t e177 t e178 t t e180 t e181 t e176 t t t t e170 e185 e179 e175 t e168 t e161 t t e190 t t e157 e143 t e138 t e194 t e135 t e196 t e133 t e198 t t e199 e131 e130 t t t t t e205 e203 t t e206 t t t t t t t e213 t t t t t e215 e214 e209 e207 t t t t t e224 t t t e226 t t t t t t t t t e232 e233 t e231 e228 e227 e225 t t t e239 t t t e242 t e244 t t t t t t t e247 e241 t e222 t t t t e255 e254 t t t t t t e260 t e221 e220 t t t t t e268 t e269 e267 e265 t t e271 t t e272 e273 e259 t e274 e258 t t e276 t t t t t t e282 t t t e284 e278 t t e287 e280 t t t t t e289 t t t t t t e293 t t e300 e299 t e296 t e294 t e291 t t t t t t t e310 t t e311 t t t t e316 t t t e319 t t t t t e324 t e320 t e318 e315 e321 t t e314 e313 t t e329 t e331 e279 e277 t t t t t t t t t t e339 e336 t t e333 t t t t t t e344 t t e351 t t e353 t t t t t t e358 t e360 t t t t t e361 e355 t t e350 e365 e348 t t e369 e346 t t e371 e343 e342 t e372 e340 e338 t e337 t t t e376 t t e378 t t t t e382 t t t t 174 [e384,e383,e377,e374,e368,e366,e364,e357,e356,e349,e347,e345,e335,e325,e322,e309,e307,e306,e295,e288,e275,e257,e256,e252,e248,e230,e219,e202,e10,e5,e386,e385,e381,e367,e359,e354,e341,e326,e323,e302,e301,e298,e297,e292,e283,e264,e263,e261,e253,e251,e249,e236,e235,e218,e217,e211,e208,e201,e197,e195,e193,e191,e188,e187,e186,e183,e173,e167,e164,e160,e156,e154,e148,e142,e141,e137,e123,e118,e111,e106,e104,e100,e96,e92,e85,e83,e79,e74,e72,e69,e49,e45,e43,e37,e33,e24,e20,e17,e380,e379,e375,e373,e370,e363,e362,e352,e334,e332,e330,e328,e327,e317,e312,e308,e305,e304,e303,e290,e286,e285,e281,e270,e266,e262,e250,e246,e245,e243,e240,e238,e237,e234,e229,e223,e216,e212,e210,e204,e200,e192,e189,e184,e182,e174,e169,e165,e163,e158,e153,e146,e136,e128,e122,e117,e113,e112,e107,e102,e99,e95,e91,e76,e75,e71,e68,e67,e62,e56,e47,e46,e35,e32,e29,e7]
```
