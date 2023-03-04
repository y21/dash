window.BENCHMARK_DATA = {
  "lastUpdate": 1677942198111,
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
      }
    ]
  }
}