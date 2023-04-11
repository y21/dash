window.BENCHMARK_DATA = {
  "lastUpdate": 1681253042404,
  "repoUrl": "https://github.com/y21/dash",
  "entries": {
    "Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "2d1f683988a9d52a8ada1335370762bf0b3d0841",
          "message": "ci: one more time",
          "timestamp": "2023-01-25T00:46:38+01:00",
          "tree_id": "a49dcc34a6b6052d24a5277bdc2083864b7ecb72",
          "url": "https://github.com/y21/dash/commit/2d1f683988a9d52a8ada1335370762bf0b3d0841"
        },
        "date": 1674604229792,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3143182,
            "range": "± 44206",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 292556,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 76349,
            "range": "± 87",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "41a628f4ea4cde326c139eb8e8e3a1b3e011429d",
          "message": "compiler: support global global post/prefix exprs",
          "timestamp": "2023-01-25T19:16:13+01:00",
          "tree_id": "ab237170302e6bfbc0692ef1b60516d18a7c95d9",
          "url": "https://github.com/y21/dash/commit/41a628f4ea4cde326c139eb8e8e3a1b3e011429d"
        },
        "date": 1674670817838,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3303598,
            "range": "± 41671",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 294749,
            "range": "± 1338",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 76838,
            "range": "± 623",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "ded3c31ba8ba9d15e7da30588a95306f043fb5bf",
          "message": "vm: implement for..in loop",
          "timestamp": "2023-01-25T20:02:08+01:00",
          "tree_id": "7f71cbc71ddc58f43ff999b33dc5d51eab72e839",
          "url": "https://github.com/y21/dash/commit/ded3c31ba8ba9d15e7da30588a95306f043fb5bf"
        },
        "date": 1674673590618,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3147517,
            "range": "± 30104",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293408,
            "range": "± 556",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 75867,
            "range": "± 129",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "f8b2ffcbef76c177952ae6f5247183c56ff0713d",
          "message": "vm: add size hints and add Float{32,64}Array",
          "timestamp": "2023-01-26T00:05:08+01:00",
          "tree_id": "764b7f44331cdf8f88271ccceedb7c3cada07011",
          "url": "https://github.com/y21/dash/commit/f8b2ffcbef76c177952ae6f5247183c56ff0713d"
        },
        "date": 1674688141543,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3297973,
            "range": "± 66519",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 287324,
            "range": "± 2565",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 77939,
            "range": "± 551",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "0bf35140d655de3463979cf21984f18d3b36170c",
          "message": "Revert boxed primitive delegation",
          "timestamp": "2023-01-26T14:18:49+01:00",
          "tree_id": "0a29977be6d924179e582f743d10588a50752b70",
          "url": "https://github.com/y21/dash/commit/0bf35140d655de3463979cf21984f18d3b36170c"
        },
        "date": 1674739432941,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 4041385,
            "range": "± 260248",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 392445,
            "range": "± 26977",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 109246,
            "range": "± 14974",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "e5fdef0f47031f6b65080eaea1ce70e0a6d801f9",
          "message": "vm: passthrough get_property",
          "timestamp": "2023-01-28T01:33:04+01:00",
          "tree_id": "4df1a5b6caa88c54d4e49cd004c327b557720638",
          "url": "https://github.com/y21/dash/commit/e5fdef0f47031f6b65080eaea1ce70e0a6d801f9"
        },
        "date": 1674866222211,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3306586,
            "range": "± 59145",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 290762,
            "range": "± 4146",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 79751,
            "range": "± 1376",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "aa1957ec221ea701af2821e25922c045e483cea0",
          "message": "handle Value::Null case in Object::get_property_descriptor",
          "timestamp": "2023-01-28T01:58:34+01:00",
          "tree_id": "e3587ff529883df30dc55a287ab5d85ab9164b34",
          "url": "https://github.com/y21/dash/commit/aa1957ec221ea701af2821e25922c045e483cea0"
        },
        "date": 1674867751449,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3363291,
            "range": "± 55919",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 290024,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 78299,
            "range": "± 357",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "e90c3aa2deb7254c9b72e34efbea384ddb328c75",
          "message": "implement TypedArray.prototype.fill",
          "timestamp": "2023-01-28T03:57:28+01:00",
          "tree_id": "eb154352f807655c65868f581705354eda30bf43",
          "url": "https://github.com/y21/dash/commit/e90c3aa2deb7254c9b72e34efbea384ddb328c75"
        },
        "date": 1674874953738,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3939227,
            "range": "± 174703",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 368840,
            "range": "± 10624",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 108588,
            "range": "± 3488",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "544d0f4e768c2e41916f88d025feb2fd61662857",
          "message": "add Date.now",
          "timestamp": "2023-01-28T04:38:58+01:00",
          "tree_id": "1aa2d8abd7c5da7505b78f73524eaef72a3c73da",
          "url": "https://github.com/y21/dash/commit/544d0f4e768c2e41916f88d025feb2fd61662857"
        },
        "date": 1674877373219,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3337887,
            "range": "± 35039",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 294124,
            "range": "± 3354",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84313,
            "range": "± 1089",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "67b14cbe58c6c31cbdbdf43a9f746be09b74439b",
          "message": "consteval: fix incorrect f64 to i32 conversion",
          "timestamp": "2023-01-28T16:45:58+01:00",
          "tree_id": "6d79e59912f2642ee22466b4bfaceb77eb6a2748",
          "url": "https://github.com/y21/dash/commit/67b14cbe58c6c31cbdbdf43a9f746be09b74439b"
        },
        "date": 1674920995074,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2772827,
            "range": "± 27329",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 256752,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81863,
            "range": "± 410",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "dcfde0e787d35eb671ef5a5f1f404ab2d11409fe",
          "message": "add Array.from",
          "timestamp": "2023-01-28T22:29:20+01:00",
          "tree_id": "caa188f184b76f3c86a2c43bac79eecfe20b89bb",
          "url": "https://github.com/y21/dash/commit/dcfde0e787d35eb671ef5a5f1f404ab2d11409fe"
        },
        "date": 1674943952346,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3655055,
            "range": "± 50513",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 338568,
            "range": "± 4538",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 98547,
            "range": "± 1562",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "50854daf7b44bb6e34c69431791109319750de7d",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/50854daf7b44bb6e34c69431791109319750de7d"
        },
        "date": 1676754496911,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3351367,
            "range": "± 170178",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293002,
            "range": "± 1652",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83144,
            "range": "± 550",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "50854daf7b44bb6e34c69431791109319750de7d",
          "message": "create function binding in the right scope",
          "timestamp": "2023-02-18T22:04:08+01:00",
          "tree_id": "eee0feb1bfb2f4e5eb298221330488d6e4669fae",
          "url": "https://github.com/y21/dash/commit/50854daf7b44bb6e34c69431791109319750de7d"
        },
        "date": 1676754510056,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3134265,
            "range": "± 5473",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 292508,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 80068,
            "range": "± 90",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "bcce29c6a858185f7f008d54d744adef3b84928f",
          "message": "compiler: deinfer external variables",
          "timestamp": "2023-02-18T23:04:46+01:00",
          "tree_id": "7de40a02efd9ea44972fd6c12eec17d5e26feb5c",
          "url": "https://github.com/y21/dash/commit/bcce29c6a858185f7f008d54d744adef3b84928f"
        },
        "date": 1676758140029,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3185130,
            "range": "± 19735",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 294534,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81703,
            "range": "± 1011",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "bcce29c6a858185f7f008d54d744adef3b84928f",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/bcce29c6a858185f7f008d54d744adef3b84928f"
        },
        "date": 1676758185537,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3714427,
            "range": "± 77694",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 336816,
            "range": "± 6199",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 95953,
            "range": "± 1170",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "16b024ce84481c1230486cb47bf3464e59dd924d",
          "message": "cfx: readd basic opts",
          "timestamp": "2023-02-19T02:52:18+01:00",
          "tree_id": "0332a462f90cff280fec9ab1d3f568e64d6a2ba4",
          "url": "https://github.com/y21/dash/commit/16b024ce84481c1230486cb47bf3464e59dd924d"
        },
        "date": 1676771782295,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3385290,
            "range": "± 37060",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 295044,
            "range": "± 2321",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83622,
            "range": "± 241",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "16b024ce84481c1230486cb47bf3464e59dd924d",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/16b024ce84481c1230486cb47bf3464e59dd924d"
        },
        "date": 1676771790208,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3395504,
            "range": "± 36072",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293006,
            "range": "± 5754",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83286,
            "range": "± 1425",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "fae7228b39387606ab205b879abf50fc3fc55d1d",
          "message": "lexer: handle offset miscalculation for owned cows",
          "timestamp": "2023-02-19T03:08:40+01:00",
          "tree_id": "eedc0dbd3330a45a93d0e1c7be4bd12f6d10142e",
          "url": "https://github.com/y21/dash/commit/fae7228b39387606ab205b879abf50fc3fc55d1d"
        },
        "date": 1676772781899,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3181070,
            "range": "± 11353",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293945,
            "range": "± 671",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82062,
            "range": "± 220",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "fae7228b39387606ab205b879abf50fc3fc55d1d",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/fae7228b39387606ab205b879abf50fc3fc55d1d"
        },
        "date": 1676772783147,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3396337,
            "range": "± 46022",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 295732,
            "range": "± 3751",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83813,
            "range": "± 239",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "f3c2de4a3b2c283c7d3500bb3f00f5b136acd6fb",
          "message": "wasm: make it compile",
          "timestamp": "2023-02-19T03:54:26+01:00",
          "tree_id": "c1e4860c9dab388c95c677c1c4dfebda94bfb526",
          "url": "https://github.com/y21/dash/commit/f3c2de4a3b2c283c7d3500bb3f00f5b136acd6fb"
        },
        "date": 1676775577939,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3954130,
            "range": "± 170878",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 363118,
            "range": "± 16988",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 112901,
            "range": "± 7770",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "f3c2de4a3b2c283c7d3500bb3f00f5b136acd6fb",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/f3c2de4a3b2c283c7d3500bb3f00f5b136acd6fb"
        },
        "date": 1676775586915,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 4163500,
            "range": "± 267854",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 380711,
            "range": "± 38244",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 109488,
            "range": "± 6394",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "e079486a97f097b7b114e96cc95869e987c087b8",
          "message": "tcx: fix typeof operator having wrong type",
          "timestamp": "2023-02-19T04:09:48+01:00",
          "tree_id": "4cc9270e152aa90bd10ca1099584f8dcff77659a",
          "url": "https://github.com/y21/dash/commit/e079486a97f097b7b114e96cc95869e987c087b8"
        },
        "date": 1676776445200,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3193943,
            "range": "± 37281",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293341,
            "range": "± 3235",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81869,
            "range": "± 119",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "e079486a97f097b7b114e96cc95869e987c087b8",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/e079486a97f097b7b114e96cc95869e987c087b8"
        },
        "date": 1676776490687,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3704770,
            "range": "± 155239",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 350040,
            "range": "± 17811",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 105220,
            "range": "± 5445",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "86bc0257757e15213c9ad40ab96a39cd0f69b89e",
          "message": "compiler: fix external value duplication checking",
          "timestamp": "2023-02-19T14:52:16+01:00",
          "tree_id": "2f07a034319e43876574055bd1db0817438a4039",
          "url": "https://github.com/y21/dash/commit/86bc0257757e15213c9ad40ab96a39cd0f69b89e"
        },
        "date": 1676814979410,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3369464,
            "range": "± 25832",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 295905,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84257,
            "range": "± 192",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "86bc0257757e15213c9ad40ab96a39cd0f69b89e",
          "message": "[WIP] run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/86bc0257757e15213c9ad40ab96a39cd0f69b89e"
        },
        "date": 1676814989774,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3392941,
            "range": "± 32615",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 300652,
            "range": "± 2612",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83707,
            "range": "± 403",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "d5641ef06db962dc2581f1e64d0701008b8cd203",
          "message": "compiler: add func jump labels in sub ib",
          "timestamp": "2023-02-19T16:53:01+01:00",
          "tree_id": "fe415c36f05ae780dda49e2cf0b41468624831c5",
          "url": "https://github.com/y21/dash/commit/d5641ef06db962dc2581f1e64d0701008b8cd203"
        },
        "date": 1676822238288,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3397742,
            "range": "± 40691",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 294199,
            "range": "± 1784",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84995,
            "range": "± 1343",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "d5641ef06db962dc2581f1e64d0701008b8cd203",
          "message": "run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/d5641ef06db962dc2581f1e64d0701008b8cd203"
        },
        "date": 1676822252083,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3250760,
            "range": "± 13325",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 295050,
            "range": "± 1251",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81820,
            "range": "± 2006",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "77829926e1e84ae42d4da09539b7bd0c15a01aaf",
          "message": "run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/77829926e1e84ae42d4da09539b7bd0c15a01aaf"
        },
        "date": 1677087942489,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3915546,
            "range": "± 281233",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 344921,
            "range": "± 18260",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 105904,
            "range": "± 5555",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "77829926e1e84ae42d4da09539b7bd0c15a01aaf",
          "message": "reimplement prefix/postfix assignment",
          "timestamp": "2023-02-22T18:40:41+01:00",
          "tree_id": "3d64d84db44384db1d69380c02c2e71f8f6f824b",
          "url": "https://github.com/y21/dash/commit/77829926e1e84ae42d4da09539b7bd0c15a01aaf"
        },
        "date": 1677087966480,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 4134266,
            "range": "± 196342",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 369873,
            "range": "± 21448",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 114685,
            "range": "± 6126",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "26f99cf2612347f8214369c941fc9053b7e1db4e",
          "message": "reimplement pre/postfix assignment for externals",
          "timestamp": "2023-02-22T19:09:19+01:00",
          "tree_id": "89896470c7281117434bd60b3b7c36f85b5dd754",
          "url": "https://github.com/y21/dash/commit/26f99cf2612347f8214369c941fc9053b7e1db4e"
        },
        "date": 1677089617120,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2820706,
            "range": "± 9155",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293353,
            "range": "± 2706",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82126,
            "range": "± 124",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "26f99cf2612347f8214369c941fc9053b7e1db4e",
          "message": "run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/26f99cf2612347f8214369c941fc9053b7e1db4e"
        },
        "date": 1677089625582,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3187491,
            "range": "± 24046",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293553,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82379,
            "range": "± 147",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "92145e71694545c4baa3d585318b87a223142922",
          "message": "fix last edge cases in spec ops",
          "timestamp": "2023-03-01T20:14:43+01:00",
          "tree_id": "80eb5899a53cdda14f6b72c129b2efa8b7b3eda4",
          "url": "https://github.com/y21/dash/commit/92145e71694545c4baa3d585318b87a223142922"
        },
        "date": 1677698360492,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3194230,
            "range": "± 27596",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 292186,
            "range": "± 754",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81092,
            "range": "± 132",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "92145e71694545c4baa3d585318b87a223142922",
          "message": "run type inference as its own pass",
          "timestamp": "2023-01-22T00:13:57Z",
          "url": "https://github.com/y21/dash/pull/50/commits/92145e71694545c4baa3d585318b87a223142922"
        },
        "date": 1677698388844,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3796597,
            "range": "± 23516",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 348312,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 96718,
            "range": "± 163",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "Timo",
            "username": "y21"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f08f7cf838cf26ed9d424471a5f012183abce555",
          "message": "Merge pull request #50 from y21/type-infer-pass\n\nrun type inference as its own pass",
          "timestamp": "2023-03-01T20:23:38+01:00",
          "tree_id": "80eb5899a53cdda14f6b72c129b2efa8b7b3eda4",
          "url": "https://github.com/y21/dash/commit/f08f7cf838cf26ed9d424471a5f012183abce555"
        },
        "date": 1677698893106,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3173730,
            "range": "± 18208",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293094,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81382,
            "range": "± 65",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "dab02bfb7ed2a5a892699afe732ae09a71dcae30",
          "message": "cast operands to u32 for ushr op",
          "timestamp": "2023-03-02T12:35:17+01:00",
          "tree_id": "de0175d80ebd5155e7f894516d6ce23293b35e0d",
          "url": "https://github.com/y21/dash/commit/dab02bfb7ed2a5a892699afe732ae09a71dcae30"
        },
        "date": 1677757244429,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3759774,
            "range": "± 131720",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 340415,
            "range": "± 7769",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 95571,
            "range": "± 400",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "b7cf5ea23f9da2754b414d30e6311b8a0d9ea107",
          "message": "testrunner: parse yaml metadata",
          "timestamp": "2023-03-02T13:04:49+01:00",
          "tree_id": "91df3592b9b734264be0d7b7527018b80d900141",
          "url": "https://github.com/y21/dash/commit/b7cf5ea23f9da2754b414d30e6311b8a0d9ea107"
        },
        "date": 1677758985715,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3170908,
            "range": "± 25335",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293640,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 79526,
            "range": "± 64",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "bdc9deeed0ae366d2e2dafa8fbd6f779be27a1fa",
          "message": "vm: throw concrete error types",
          "timestamp": "2023-03-02T13:47:19+01:00",
          "tree_id": "dfa36dcf526bc0c29cb9e7da891767786784cb82",
          "url": "https://github.com/y21/dash/commit/bdc9deeed0ae366d2e2dafa8fbd6f779be27a1fa"
        },
        "date": 1677761514256,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3186175,
            "range": "± 54226",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 295815,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82143,
            "range": "± 201",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "04c6b646e71c79354d393338d7b9e86348205a9d",
          "message": "pass through suberror ctor/proto",
          "timestamp": "2023-03-02T14:09:02+01:00",
          "tree_id": "411202febc664c77e14dfa59fc607965fa1cb1bf",
          "url": "https://github.com/y21/dash/commit/04c6b646e71c79354d393338d7b9e86348205a9d"
        },
        "date": 1677762817244,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3422911,
            "range": "± 69590",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 298188,
            "range": "± 6096",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84296,
            "range": "± 842",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "c643aff764eafcad52d3f7b879eb3042d6524537",
          "message": "testrunner: support negative tests",
          "timestamp": "2023-03-02T15:00:57+01:00",
          "tree_id": "0698a5e1a7d808e4cad662265e75293ed80b990a",
          "url": "https://github.com/y21/dash/commit/c643aff764eafcad52d3f7b879eb3042d6524537"
        },
        "date": 1677765936507,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3433765,
            "range": "± 60793",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 299154,
            "range": "± 5646",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84111,
            "range": "± 401",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "50f5a6576631f85b935f2fe1032da91f245246ff",
          "message": "handle expected_args != actual_args case in generators",
          "timestamp": "2023-03-04T14:59:38+01:00",
          "tree_id": "6ffe7b8897e5992c188b9612af4eafba7f105fd9",
          "url": "https://github.com/y21/dash/commit/50f5a6576631f85b935f2fe1032da91f245246ff"
        },
        "date": 1677938644951,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3407055,
            "range": "± 27101",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 293105,
            "range": "± 1868",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82184,
            "range": "± 1347",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "deb63a0886e719eaeb91e791a828647f3a7a80e6",
          "message": "lexer: support 1e-1 syntax",
          "timestamp": "2023-03-04T15:23:05+01:00",
          "tree_id": "139d3f4ee4f1230a5d919fa23a15bb9e56ff5c3c",
          "url": "https://github.com/y21/dash/commit/deb63a0886e719eaeb91e791a828647f3a7a80e6"
        },
        "date": 1677940128014,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3416707,
            "range": "± 102669",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 291422,
            "range": "± 2956",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83024,
            "range": "± 94",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "6b424cee77886cf2da37031df6735b74fad1b19f",
          "message": "parser: handle export <invalid> case",
          "timestamp": "2023-03-04T15:58:48+01:00",
          "tree_id": "d8093dfb50ab73f4cf1224b40f9c25aa32580fd9",
          "url": "https://github.com/y21/dash/commit/6b424cee77886cf2da37031df6735b74fad1b19f"
        },
        "date": 1677942196898,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3424381,
            "range": "± 54082",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 289267,
            "range": "± 1803",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82443,
            "range": "± 1962",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "939ce9478a9d128555c946fb05de055c04eaed80",
          "message": "only do stack size checking when pushing a frame",
          "timestamp": "2023-03-04T17:09:02+01:00",
          "tree_id": "3f89311c8528ba09b5a2f6cd92cf7cbc8d213e77",
          "url": "https://github.com/y21/dash/commit/939ce9478a9d128555c946fb05de055c04eaed80"
        },
        "date": 1677946403337,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3316869,
            "range": "± 42697",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 278949,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82017,
            "range": "± 269",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "7b678f900a4e3378ad9f285748ded72115a7b6a8",
          "message": "jit: implement BB discovery pass",
          "timestamp": "2023-03-06T20:38:46+01:00",
          "tree_id": "b563306f5a43dfb113bcd41984a122a0d7045535",
          "url": "https://github.com/y21/dash/commit/7b678f900a4e3378ad9f285748ded72115a7b6a8"
        },
        "date": 1678131803963,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3315492,
            "range": "± 23153",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 279636,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83204,
            "range": "± 1095",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "2ba7b5eda7bfb021a4476c31d1b106cb4dabbc9d",
          "message": "jit: type infer pass based on BBs",
          "timestamp": "2023-03-07T10:41:13+01:00",
          "tree_id": "89f4acdf9e6f125c368307fcd109079d3dc53155",
          "url": "https://github.com/y21/dash/commit/2ba7b5eda7bfb021a4476c31d1b106cb4dabbc9d"
        },
        "date": 1678188553092,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3781494,
            "range": "± 218039",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 309340,
            "range": "± 7469",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 92380,
            "range": "± 2308",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "2dbb6c26fea98a65fc4de3dc838d286512a4a0ab",
          "message": "jit: impl codegenctxt",
          "timestamp": "2023-03-08T17:31:28+01:00",
          "tree_id": "29ae0704077d2208a9932deec91903a43e6d06fd",
          "url": "https://github.com/y21/dash/commit/2dbb6c26fea98a65fc4de3dc838d286512a4a0ab"
        },
        "date": 1678293823891,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3389971,
            "range": "± 126349",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 315702,
            "range": "± 17873",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 99524,
            "range": "± 3994",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "583c671de0b4b7ab7d5196dd0ec7261a8a5f54c3",
          "message": "jit: finish subtraces",
          "timestamp": "2023-03-14T00:25:15+01:00",
          "tree_id": "1471b1b33454943205889eceee2e6d12d83d3294",
          "url": "https://github.com/y21/dash/commit/583c671de0b4b7ab7d5196dd0ec7261a8a5f54c3"
        },
        "date": 1678750205191,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3327286,
            "range": "± 32304",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 277507,
            "range": "± 1658",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84590,
            "range": "± 99",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "dc5f19668ef3085234df35d3eb5d28a67f76a05b",
          "message": "jit: fix ValueStack::pop2 order",
          "timestamp": "2023-03-14T22:26:41+01:00",
          "tree_id": "1589a009e66a84ae6e91caa0ebe1847570a12f5c",
          "url": "https://github.com/y21/dash/commit/dc5f19668ef3085234df35d3eb5d28a67f76a05b"
        },
        "date": 1678829486450,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3106485,
            "range": "± 15175",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 272415,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82194,
            "range": "± 82",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "f7d224a1319f92b1603a806b973d01a4de93dc12",
          "message": "jit: assert that valuestack is empty",
          "timestamp": "2023-03-15T01:13:16+01:00",
          "tree_id": "7562a59529b3e186943477e15b1163807b9f4766",
          "url": "https://github.com/y21/dash/commit/f7d224a1319f92b1603a806b973d01a4de93dc12"
        },
        "date": 1678839548147,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3658417,
            "range": "± 64140",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 319591,
            "range": "± 4239",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 94503,
            "range": "± 2071",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "5485af8e965e60d07d0d0e760108810c5a0741b9",
          "message": "jit: simplify cache logic",
          "timestamp": "2023-03-15T20:56:48+01:00",
          "tree_id": "fce84e4e62c3b04740136b8912ddccb81aa30753",
          "url": "https://github.com/y21/dash/commit/5485af8e965e60d07d0d0e760108810c5a0741b9"
        },
        "date": 1678910549463,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3660459,
            "range": "± 37704",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 313372,
            "range": "± 5895",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 93742,
            "range": "± 1942",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "5485af8e965e60d07d0d0e760108810c5a0741b9",
          "message": "add cfg pass to jit",
          "timestamp": "2023-03-10T08:04:16Z",
          "url": "https://github.com/y21/dash/pull/51/commits/5485af8e965e60d07d0d0e760108810c5a0741b9"
        },
        "date": 1678910803485,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3327182,
            "range": "± 47321",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 281196,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82799,
            "range": "± 335",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "Timo",
            "username": "y21"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fd526330e20bad6974447ea6c167153607cc7d0e",
          "message": "Merge pull request #51 from y21/jit-bb\n\nadd cfg pass to jit",
          "timestamp": "2023-03-15T21:02:01+01:00",
          "tree_id": "fce84e4e62c3b04740136b8912ddccb81aa30753",
          "url": "https://github.com/y21/dash/commit/fd526330e20bad6974447ea6c167153607cc7d0e"
        },
        "date": 1678910807193,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3103377,
            "range": "± 24172",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 272268,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81496,
            "range": "± 424",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "3a987c2ab8e99227ead3ca100450a06e57eef6fb",
          "message": "add typed_cfg crate to workspace",
          "timestamp": "2023-03-16T16:27:57+01:00",
          "tree_id": "fd2ec8151dec9bc67b8ba247f0e31390e3e0c627",
          "url": "https://github.com/y21/dash/commit/3a987c2ab8e99227ead3ca100450a06e57eef6fb"
        },
        "date": 1678980874811,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3585267,
            "range": "± 96161",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 306124,
            "range": "± 17594",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 92963,
            "range": "± 3857",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "573686f288c45fb6a94a9773f1dae879bf2b8f28",
          "message": "use ahash",
          "timestamp": "2023-03-16T16:37:23+01:00",
          "tree_id": "7f0aadc3b798bc4bf4f7f507830b341bb80df242",
          "url": "https://github.com/y21/dash/commit/573686f288c45fb6a94a9773f1dae879bf2b8f28"
        },
        "date": 1678981330697,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3247214,
            "range": "± 53649",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 278070,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83925,
            "range": "± 148",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "a6d3cdee658fc1d3eefc616b11ff810c0e8de4b0",
          "message": "implement do..while loops",
          "timestamp": "2023-03-21T21:01:05+01:00",
          "tree_id": "a897b21d7d565a9fa938f5b66d23754340afc420",
          "url": "https://github.com/y21/dash/commit/a6d3cdee658fc1d3eefc616b11ff810c0e8de4b0"
        },
        "date": 1679429168217,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3040465,
            "range": "± 15471",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 274256,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81290,
            "range": "± 79",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "7d78570101821076ca9092166c3b694b09bacabc",
          "message": "fix break inside of do..while loops",
          "timestamp": "2023-03-21T23:49:14+01:00",
          "tree_id": "cbc8d1ef23fce56b4fdfe7ff8a1f191714a18741",
          "url": "https://github.com/y21/dash/commit/7d78570101821076ca9092166c3b694b09bacabc"
        },
        "date": 1679439309154,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3586490,
            "range": "± 46237",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 332531,
            "range": "± 28616",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 95622,
            "range": "± 853",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "560c34d33757ddbcc0b04ac8ea89f80a17da9487",
          "message": "parser: support object method syntax",
          "timestamp": "2023-03-22T14:12:53+01:00",
          "tree_id": "149faa793b83980a7c0f1e1883698be888ab31c7",
          "url": "https://github.com/y21/dash/commit/560c34d33757ddbcc0b04ac8ea89f80a17da9487"
        },
        "date": 1679491134770,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3608260,
            "range": "± 26192",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 321118,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 95627,
            "range": "± 140",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "5a90278023d4a15a236b7e55837e115372bd8751",
          "message": "rt: make all modules opt-in through features",
          "timestamp": "2023-03-22T14:29:51+01:00",
          "tree_id": "98485c3869a11f55688d7b65318e284984e96190",
          "url": "https://github.com/y21/dash/commit/5a90278023d4a15a236b7e55837e115372bd8751"
        },
        "date": 1679492077569,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3237955,
            "range": "± 31254",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 278847,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82480,
            "range": "± 107",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "059cc81e7d972e497e72ed3be171465e61f462cc",
          "message": "clippy",
          "timestamp": "2023-03-23T17:12:54+01:00",
          "tree_id": "ee47de327956809463b27f0e9dee27eb3e21b229",
          "url": "https://github.com/y21/dash/commit/059cc81e7d972e497e72ed3be171465e61f462cc"
        },
        "date": 1679588670435,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3914664,
            "range": "± 308003",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 360043,
            "range": "± 22266",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 112224,
            "range": "± 10489",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "82e79b6cf2748dc467a16670f4e3a9faaa8935fb",
          "message": "fix compile errors",
          "timestamp": "2023-03-29T19:14:19+02:00",
          "tree_id": "b0453426674a1209ca921f65a20ac91171bc2f48",
          "url": "https://github.com/y21/dash/commit/82e79b6cf2748dc467a16670f4e3a9faaa8935fb"
        },
        "date": 1680110666445,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3699040,
            "range": "± 195872",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "Timo",
            "username": "y21"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5a7ebd0249a00ada8bbdbbfe935dbe025a467723",
          "message": "Merge pull request #53 from p2js/patch-1\n\nReturn strings in quotes regardless of depth",
          "timestamp": "2023-04-11T21:41:10+02:00",
          "tree_id": "6fde15b3d7e1def2e617ec1ded424749fd93613e",
          "url": "https://github.com/y21/dash/commit/5a7ebd0249a00ada8bbdbbfe935dbe025a467723"
        },
        "date": 1681242355008,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3705549,
            "range": "± 181629",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 351396,
            "range": "± 21067",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 111241,
            "range": "± 4816",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "95ea8bc3f00a27812e851c7b22d5d0739ae94c0f",
          "message": "fix test",
          "timestamp": "2023-04-12T00:21:20+02:00",
          "tree_id": "59b3b29f1dd457f5f795ac313f13fdaf842cea17",
          "url": "https://github.com/y21/dash/commit/95ea8bc3f00a27812e851c7b22d5d0739ae94c0f"
        },
        "date": 1681251916758,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3044931,
            "range": "± 43021",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 265693,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 73455,
            "range": "± 915",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "email": "30553356+y21@users.noreply.github.com",
            "name": "y21",
            "username": "y21"
          },
          "distinct": true,
          "id": "aad7fbf613da67a963b03be8db5dd7955e8b40a1",
          "message": "s/gc2/gc",
          "timestamp": "2023-04-12T00:30:11+02:00",
          "tree_id": "184bbff8bd73771a1ba083ff91acab5010ba6013",
          "url": "https://github.com/y21/dash/commit/aad7fbf613da67a963b03be8db5dd7955e8b40a1"
        },
        "date": 1681252439898,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3012667,
            "range": "± 35868",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 263239,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 73074,
            "range": "± 309",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "y21",
            "username": "y21"
          },
          "committer": {
            "name": "y21",
            "username": "y21"
          },
          "id": "aad7fbf613da67a963b03be8db5dd7955e8b40a1",
          "message": "rework gc to remove double indirection",
          "timestamp": "2023-04-09T15:08:27Z",
          "url": "https://github.com/y21/dash/pull/54/commits/aad7fbf613da67a963b03be8db5dd7955e8b40a1"
        },
        "date": 1681253040483,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3189322,
            "range": "± 44391",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 269645,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 76411,
            "range": "± 118",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}