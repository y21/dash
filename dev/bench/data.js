window.BENCHMARK_DATA = {
  "lastUpdate": 1721167119523,
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
          "id": "25de253b8fc42b6cb77296f88f6db0ccb8eae319",
          "message": "rework gc to remove double indirection",
          "timestamp": "2023-04-09T15:08:27Z",
          "url": "https://github.com/y21/dash/pull/54/commits/25de253b8fc42b6cb77296f88f6db0ccb8eae319"
        },
        "date": 1681259991133,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3228315,
            "range": "± 45521",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 268452,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 75903,
            "range": "± 137",
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
          "id": "25de253b8fc42b6cb77296f88f6db0ccb8eae319",
          "message": "do not rebox already boxed values",
          "timestamp": "2023-04-12T02:36:02+02:00",
          "tree_id": "dec283b09b81ef1d9034e1eae1fefc7e60f238be",
          "url": "https://github.com/y21/dash/commit/25de253b8fc42b6cb77296f88f6db0ccb8eae319"
        },
        "date": 1681260033726,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3083511,
            "range": "± 37120",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 263775,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 71592,
            "range": "± 162",
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
          "id": "8083d95d45f97150322778f1e313c5392eefe34d",
          "message": "Merge pull request #54 from y21/gc-single-indirection\n\nrework gc to remove double indirection",
          "timestamp": "2023-04-12T02:44:32+02:00",
          "tree_id": "b87334b1cbde7d3479f582707148d35e64e5bb50",
          "url": "https://github.com/y21/dash/commit/8083d95d45f97150322778f1e313c5392eefe34d"
        },
        "date": 1681260492883,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3222915,
            "range": "± 43992",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 270155,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 75225,
            "range": "± 164",
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
          "id": "b079ad8e84a355dce5dec33ea55b6a4161307faa",
          "message": "rework gc to remove double indirection",
          "timestamp": "2023-04-09T15:08:27Z",
          "url": "https://github.com/y21/dash/pull/54/commits/b079ad8e84a355dce5dec33ea55b6a4161307faa"
        },
        "date": 1681260502258,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3662873,
            "range": "± 129814",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 315217,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 87331,
            "range": "± 109",
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
          "id": "b079ad8e84a355dce5dec33ea55b6a4161307faa",
          "message": "\"hotfix\" for testrunner crashes",
          "timestamp": "2023-04-12T02:43:41+02:00",
          "tree_id": "7c546129fc8a67a70416a976a29b93b4e8ae1b86",
          "url": "https://github.com/y21/dash/commit/b079ad8e84a355dce5dec33ea55b6a4161307faa"
        },
        "date": 1681260510512,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3490571,
            "range": "± 173324",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 325631,
            "range": "± 20237",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 94513,
            "range": "± 6287",
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
          "id": "5ef80b0f0125d4275bc6476381fb22eb5255932a",
          "message": "format numbers in scientific notation",
          "timestamp": "2023-04-12T03:01:37+02:00",
          "tree_id": "110ab4bf9c7c579d89793ec2d7e0a6eeb7c69e3f",
          "url": "https://github.com/y21/dash/commit/5ef80b0f0125d4275bc6476381fb22eb5255932a"
        },
        "date": 1681261594169,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3566632,
            "range": "± 30659",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 312130,
            "range": "± 2115",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88157,
            "range": "± 589",
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
          "id": "4a2c500f1a41a890f9ec5b497495ec9f143d8e0b",
          "message": "properly format f64",
          "timestamp": "2023-04-12T15:11:07+02:00",
          "tree_id": "22bad7a8992ed558ac288a7a4fd9d77c5c162f9f",
          "url": "https://github.com/y21/dash/commit/4a2c500f1a41a890f9ec5b497495ec9f143d8e0b"
        },
        "date": 1681305309570,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3259103,
            "range": "± 48503",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 274238,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 77195,
            "range": "± 110",
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
          "id": "9ce29ecec5add40d0b2ab44c32382ff6d3a616f1",
          "message": "don't update test262",
          "timestamp": "2023-04-12T15:26:12+02:00",
          "tree_id": "5e79a2034a6a1f1ddd4f485d7a8d3d5ec127c17a",
          "url": "https://github.com/y21/dash/commit/9ce29ecec5add40d0b2ab44c32382ff6d3a616f1"
        },
        "date": 1681306220998,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3048707,
            "range": "± 14861",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 265171,
            "range": "± 817",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 73124,
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
          "id": "b01bbbe3cd8d0a23b55b2c450a4caf6b0dc7f788",
          "message": "handle non-ascii meta escapes",
          "timestamp": "2023-04-12T17:19:30+02:00",
          "tree_id": "008ded321e2909e2640559f3b5806f4a17396c20",
          "url": "https://github.com/y21/dash/commit/b01bbbe3cd8d0a23b55b2c450a4caf6b0dc7f788"
        },
        "date": 1681313075002,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3976075,
            "range": "± 192906",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 360250,
            "range": "± 22936",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 108636,
            "range": "± 6084",
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
          "id": "786a32ef0e2b3529a3d7af79ecf7614b848ae2c8",
          "message": "update deps, it's that time again",
          "timestamp": "2023-04-12T17:35:05+02:00",
          "tree_id": "26fac0b420c018838cb459571217349684695151",
          "url": "https://github.com/y21/dash/commit/786a32ef0e2b3529a3d7af79ecf7614b848ae2c8"
        },
        "date": 1681313947649,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3252754,
            "range": "± 39781",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 271888,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 76116,
            "range": "± 125",
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
          "id": "f9e7fc07ed6450c1a9b99943d3792def6514232a",
          "message": "implement flat calls",
          "timestamp": "2023-04-14T23:23:51+02:00",
          "tree_id": "324f43ff03d56bb02e65ac1888e1127e97bcdb60",
          "url": "https://github.com/y21/dash/commit/f9e7fc07ed6450c1a9b99943d3792def6514232a"
        },
        "date": 1681507676920,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2608171,
            "range": "± 11092",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 203521,
            "range": "± 1853",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 72474,
            "range": "± 57",
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
          "id": "f9e7fc07ed6450c1a9b99943d3792def6514232a",
          "message": "don't use recursion for executing functions in VM land",
          "timestamp": "2023-04-09T15:08:27Z",
          "url": "https://github.com/y21/dash/pull/55/commits/f9e7fc07ed6450c1a9b99943d3792def6514232a"
        },
        "date": 1681508176059,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2594250,
            "range": "± 4846",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 199720,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 72262,
            "range": "± 273",
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
          "id": "748cf0af6651d7fbeee4e78eac327056695a7852",
          "message": "Merge pull request #55 from y21/flat-recursion\n\ndon't use recursion for executing functions in VM land",
          "timestamp": "2023-04-14T23:32:42+02:00",
          "tree_id": "324f43ff03d56bb02e65ac1888e1127e97bcdb60",
          "url": "https://github.com/y21/dash/commit/748cf0af6651d7fbeee4e78eac327056695a7852"
        },
        "date": 1681508189659,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2825741,
            "range": "± 57976",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 215866,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 74947,
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
          "id": "2661e6f5cdf9490f166734dfed37d993ded7e31f",
          "message": "implement Map and fix some Set tests",
          "timestamp": "2023-04-15T03:27:36+02:00",
          "tree_id": "c86a10ae6fcebf369c353138dc71b3b158ffde17",
          "url": "https://github.com/y21/dash/commit/2661e6f5cdf9490f166734dfed37d993ded7e31f"
        },
        "date": 1681522293168,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2622482,
            "range": "± 27606",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 203348,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 74884,
            "range": "± 58",
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
          "id": "2661e6f5cdf9490f166734dfed37d993ded7e31f",
          "message": "implement `Map` and fix some `Set` tests",
          "timestamp": "2023-04-14T21:40:49Z",
          "url": "https://github.com/y21/dash/pull/56/commits/2661e6f5cdf9490f166734dfed37d993ded7e31f"
        },
        "date": 1681522422681,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2600180,
            "range": "± 48980",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 202219,
            "range": "± 1525",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 74902,
            "range": "± 3204",
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
          "id": "1be303d6f99f0a0004ef4feb08afd4f04c47d581",
          "message": "Merge pull request #56 from y21/feature/map\n\nimplement `Map` and fix some `Set` tests",
          "timestamp": "2023-04-15T03:29:58+02:00",
          "tree_id": "c86a10ae6fcebf369c353138dc71b3b158ffde17",
          "url": "https://github.com/y21/dash/commit/1be303d6f99f0a0004ef4feb08afd4f04c47d581"
        },
        "date": 1681522484011,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3048305,
            "range": "± 115002",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 252595,
            "range": "± 9972",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 104081,
            "range": "± 9767",
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
          "id": "0263f16b4743174a69d43129a6e15d6c676aca38",
          "message": "don't unwrap frame overflows",
          "timestamp": "2023-04-18T23:10:06+02:00",
          "tree_id": "8840b8b4f57a0d49b1e896f980f41611c6102995",
          "url": "https://github.com/y21/dash/commit/0263f16b4743174a69d43129a6e15d6c676aca38"
        },
        "date": 1681852537294,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3070364,
            "range": "± 63987",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 241127,
            "range": "± 5498",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 86736,
            "range": "± 2421",
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
          "id": "605db2eb69e335c78a98e9607a14e5aee6b62360",
          "message": "Merge branch 'master' of https://github.com/y21/dash",
          "timestamp": "2023-04-24T13:28:20+02:00",
          "tree_id": "6a4f20280ceaa8318c21a2e2ec6e47e94ad8802f",
          "url": "https://github.com/y21/dash/commit/605db2eb69e335c78a98e9607a14e5aee6b62360"
        },
        "date": 1682336003339,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3071796,
            "range": "± 196294",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 247213,
            "range": "± 14489",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 101244,
            "range": "± 5594",
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
          "id": "020a8ae2c6f334386074470bf99fb75024a59dd0",
          "message": "Merge pull request #60 from trueharuu/patch-1\n\nResolves #58",
          "timestamp": "2023-04-29T03:05:56+02:00",
          "tree_id": "85a5161ddb8885d682636d3d9e246cb2d22c2119",
          "url": "https://github.com/y21/dash/commit/020a8ae2c6f334386074470bf99fb75024a59dd0"
        },
        "date": 1682730663328,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3180765,
            "range": "± 120056",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 261498,
            "range": "± 10860",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 107023,
            "range": "± 3972",
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
          "id": "ec1dd46bda72c34e6e22b3ef28a7633d9776b21b",
          "message": "rework dispatch to only create one localscope",
          "timestamp": "2023-05-06T18:46:04+02:00",
          "tree_id": "ab7d9f7347e6349b6eafa37d0c1d6641f96ea30d",
          "url": "https://github.com/y21/dash/commit/ec1dd46bda72c34e6e22b3ef28a7633d9776b21b"
        },
        "date": 1683391870074,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3210609,
            "range": "± 29943",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 266704,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 92472,
            "range": "± 351",
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
          "id": "44df428fc2ade70ba0e08daabea57dffda390349",
          "message": "don't replace existing externals",
          "timestamp": "2023-05-07T01:19:35+02:00",
          "tree_id": "f30028444f0f7740ed10ae24765f31c0db220f8a",
          "url": "https://github.com/y21/dash/commit/44df428fc2ade70ba0e08daabea57dffda390349"
        },
        "date": 1684411935102,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2900435,
            "range": "± 9074",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 241126,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 80375,
            "range": "± 151",
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
          "id": "382fcd812e4baf1740cb6b95fd0443a66d5b5870",
          "message": "fix GC unsoundness, part 1",
          "timestamp": "2023-05-29T16:42:51+02:00",
          "tree_id": "98cd4de5e3032225e4ba480e448767d50083f24a",
          "url": "https://github.com/y21/dash/commit/382fcd812e4baf1740cb6b95fd0443a66d5b5870"
        },
        "date": 1685371915686,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3329720,
            "range": "± 35532",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 276662,
            "range": "± 2438",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 91814,
            "range": "± 739",
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
          "id": "323828667ebd956ab6739391ff5571991243331d",
          "message": "migrate from old pop_stack to new pop_stack",
          "timestamp": "2023-05-29T20:30:15+02:00",
          "tree_id": "bbd1b676e3c3548570662fcf1d1250c03c9f5fdf",
          "url": "https://github.com/y21/dash/commit/323828667ebd956ab6739391ff5571991243331d"
        },
        "date": 1685385312092,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3037832,
            "range": "± 20969",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 253713,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81712,
            "range": "± 158",
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
          "id": "fc2570512cf4a67790c5578950329bca2de6c788",
          "message": "parse type annotations in return position",
          "timestamp": "2023-07-09T18:12:21+02:00",
          "tree_id": "b60710d4efaae8e35c2e2d05bf20b9ef2b563ff9",
          "url": "https://github.com/y21/dash/commit/fc2570512cf4a67790c5578950329bca2de6c788"
        },
        "date": 1688919350032,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2603309,
            "range": "± 166003",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 198046,
            "range": "± 783",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 76338,
            "range": "± 246",
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
          "id": "281d72708171cf38eb55fff98df92c559d9a7fa6",
          "message": "fix clippy warnings and use Unrooted API in more places",
          "timestamp": "2023-07-31T22:04:43+02:00",
          "tree_id": "a7a47b5503a11d96cb0251c718c0aadeb796fe33",
          "url": "https://github.com/y21/dash/commit/281d72708171cf38eb55fff98df92c559d9a7fa6"
        },
        "date": 1690834097260,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2956821,
            "range": "± 40810",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 229846,
            "range": "± 760",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 76357,
            "range": "± 66",
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
          "id": "281d72708171cf38eb55fff98df92c559d9a7fa6",
          "message": "introduce `Unrooted` API",
          "timestamp": "2023-04-14T21:40:49Z",
          "url": "https://github.com/y21/dash/pull/67/commits/281d72708171cf38eb55fff98df92c559d9a7fa6"
        },
        "date": 1690835753437,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3097862,
            "range": "± 56588",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 230328,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 79735,
            "range": "± 283",
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
          "id": "b5c1b48006552f34b46e6c9c9c30a9d4c07ea331",
          "message": "Merge pull request #67 from y21/gc-soundness\n\nintroduce `Unrooted` API",
          "timestamp": "2023-07-31T22:32:28+02:00",
          "tree_id": "7a88f9c2938097fb3482fabdff951cf29f2a336a",
          "url": "https://github.com/y21/dash/commit/b5c1b48006552f34b46e6c9c9c30a9d4c07ea331"
        },
        "date": 1690835757550,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2958426,
            "range": "± 9163",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 229150,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 78382,
            "range": "± 339",
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
          "id": "e8cb2bfdc5070ad7edba5cff218d6a2c743d1201",
          "message": "use inspect in cli",
          "timestamp": "2023-08-01T01:22:58+02:00",
          "tree_id": "18f6653045aeb729cb4401f2c5acaf35e9f30672",
          "url": "https://github.com/y21/dash/commit/e8cb2bfdc5070ad7edba5cff218d6a2c743d1201"
        },
        "date": 1690846038919,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3586701,
            "range": "± 196437",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 280212,
            "range": "± 18552",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 105707,
            "range": "± 6344",
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
          "id": "b0019fa91765f01858a11a4685aaa358649f441c",
          "message": "recursively visit prototypes for `instanceof` check",
          "timestamp": "2023-08-01T18:59:02+02:00",
          "tree_id": "e43556d23e7300d57a7234486364d4cadabf9e98",
          "url": "https://github.com/y21/dash/commit/b0019fa91765f01858a11a4685aaa358649f441c"
        },
        "date": 1690909354575,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2831235,
            "range": "± 66726",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 229555,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 77145,
            "range": "± 341",
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
          "id": "8284c55f8e7b0f614ff00575297fe4caff12a5a3",
          "message": "fix CI",
          "timestamp": "2023-08-01T19:57:03+02:00",
          "tree_id": "a2230300d3c60affc32d4eeb8fa65a1cf78563ac",
          "url": "https://github.com/y21/dash/commit/8284c55f8e7b0f614ff00575297fe4caff12a5a3"
        },
        "date": 1690912825893,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3111022,
            "range": "± 14101",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 230939,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 80404,
            "range": "± 325",
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
          "id": "87b3994fd10305948a0bce3bbe7263762886ea29",
          "message": "fix incorrect AwaitOutsideAsync",
          "timestamp": "2023-08-01T23:47:45+02:00",
          "tree_id": "ba3a2e1f90ded0d66ba042b4f657f3178de032bb",
          "url": "https://github.com/y21/dash/commit/87b3994fd10305948a0bce3bbe7263762886ea29"
        },
        "date": 1690926719178,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3755425,
            "range": "± 194305",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 306612,
            "range": "± 10350",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 111912,
            "range": "± 3042",
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
          "id": "78816494362f91958d24a01a682a052c8a10e12a",
          "message": "implement `@std/net`, accepting+writing works",
          "timestamp": "2023-08-02T01:21:03+02:00",
          "tree_id": "f3a242f48dd38ad7a11c06743b3d89244d16643f",
          "url": "https://github.com/y21/dash/commit/78816494362f91958d24a01a682a052c8a10e12a"
        },
        "date": 1690932270445,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2669264,
            "range": "± 133320",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 226224,
            "range": "± 7356",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 85327,
            "range": "± 5728",
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
          "id": "313d7cf23123b24ea4e5ffb354e22d30d9324932",
          "message": "net: implement receiving",
          "timestamp": "2023-08-02T19:27:21+02:00",
          "tree_id": "4389ff5d677774e47945e00216efb5fdace41c9c",
          "url": "https://github.com/y21/dash/commit/313d7cf23123b24ea4e5ffb354e22d30d9324932"
        },
        "date": 1690997482174,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3526456,
            "range": "± 24810",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 274249,
            "range": "± 1881",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 91637,
            "range": "± 692",
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
          "id": "95291541ba06f1a0c4107e46120e64006ef27299",
          "message": "trace currently-used scopes",
          "timestamp": "2023-08-03T00:57:12+02:00",
          "tree_id": "9add5e2340c29ffba96660a27d0854cd828e28e9",
          "url": "https://github.com/y21/dash/commit/95291541ba06f1a0c4107e46120e64006ef27299"
        },
        "date": 1691017264991,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2901599,
            "range": "± 182498",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 255158,
            "range": "± 21450",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 98705,
            "range": "± 8076",
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
          "id": "66d1f9ac93be2cf058f7c893421c29f1bd4202b8",
          "message": "consider external refs as roots",
          "timestamp": "2023-08-03T20:31:05+02:00",
          "tree_id": "876ab8942d84a6624a352954ab0a702ed487c4b3",
          "url": "https://github.com/y21/dash/commit/66d1f9ac93be2cf058f7c893421c29f1bd4202b8"
        },
        "date": 1691087714102,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3037384,
            "range": "± 97702",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 237404,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 79692,
            "range": "± 353",
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
          "id": "e75f48f1c6a2208980655b6295bbe58a86f18adc",
          "message": "final cleanups",
          "timestamp": "2023-08-03T20:33:56+02:00",
          "tree_id": "2896bd609804d0273223136fce72e234b79f30f5",
          "url": "https://github.com/y21/dash/commit/e75f48f1c6a2208980655b6295bbe58a86f18adc"
        },
        "date": 1691087846696,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3128713,
            "range": "± 64513",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 229570,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 79516,
            "range": "± 213",
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
          "id": "e75f48f1c6a2208980655b6295bbe58a86f18adc",
          "message": "implement net module and consider external refs as roots",
          "timestamp": "2023-04-14T21:40:49Z",
          "url": "https://github.com/y21/dash/pull/69/commits/e75f48f1c6a2208980655b6295bbe58a86f18adc"
        },
        "date": 1691087937498,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3495730,
            "range": "± 223462",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 273782,
            "range": "± 3352",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 91747,
            "range": "± 2150",
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
          "id": "bfb52b2dc975964ce11689953f33a61fa1714060",
          "message": "Merge pull request #69 from y21/net\n\nimplement net module and consider external refs as roots",
          "timestamp": "2023-08-03T20:35:13+02:00",
          "tree_id": "2896bd609804d0273223136fce72e234b79f30f5",
          "url": "https://github.com/y21/dash/commit/bfb52b2dc975964ce11689953f33a61fa1714060"
        },
        "date": 1691087948272,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3521921,
            "range": "± 59166",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 269861,
            "range": "± 1529",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 90994,
            "range": "± 1045",
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
          "id": "9b3302f070a1d8c873036b57caee53033547101c",
          "message": "make more things `Unrooted`",
          "timestamp": "2023-08-03T22:42:31+02:00",
          "tree_id": "b9ca349c073f7b6048186ac04b1c073921414949",
          "url": "https://github.com/y21/dash/commit/9b3302f070a1d8c873036b57caee53033547101c"
        },
        "date": 1691095631018,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3845140,
            "range": "± 187162",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 303039,
            "range": "± 10563",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 112008,
            "range": "± 5233",
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
          "id": "e24fc2b69f1ddf9cc5b1be60f813cb462f23d2f1",
          "message": "fix CI error",
          "timestamp": "2023-08-04T21:16:16+02:00",
          "tree_id": "e25b61de19bea4c95d3dd52e25ebf56bef105fd7",
          "url": "https://github.com/y21/dash/commit/e24fc2b69f1ddf9cc5b1be60f813cb462f23d2f1"
        },
        "date": 1691176786320,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3020634,
            "range": "± 73710",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 236793,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 77783,
            "range": "± 1090",
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
          "id": "926ebe8d8391b0f75cd551207c8ea17810f52572",
          "message": "add `JSON.parse`",
          "timestamp": "2023-08-06T17:04:35+02:00",
          "tree_id": "6e7c57e7ffbb6d3d5a15b8e4861ef89db5cb373b",
          "url": "https://github.com/y21/dash/commit/926ebe8d8391b0f75cd551207c8ea17810f52572"
        },
        "date": 1691334483054,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3002668,
            "range": "± 42081",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 247032,
            "range": "± 8161",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 78098,
            "range": "± 971",
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
          "id": "59fa159df5a3cd938f520b9a51ebdade8600343b",
          "message": "hoist class bindings",
          "timestamp": "2023-08-08T00:06:20+02:00",
          "tree_id": "53a300ea5832cdb795c576ce2ec2d5761aaa2e32",
          "url": "https://github.com/y21/dash/commit/59fa159df5a3cd938f520b9a51ebdade8600343b"
        },
        "date": 1691446191420,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3004848,
            "range": "± 4202",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 238904,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 78525,
            "range": "± 631",
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
          "id": "b52e67d4972b02a377badcdab4694870f3e0054d",
          "message": "add `Object.defineProperty`",
          "timestamp": "2023-08-08T02:15:13+02:00",
          "tree_id": "6238da17f504bb6349c7bbe1e9745d34c26e1ea1",
          "url": "https://github.com/y21/dash/commit/b52e67d4972b02a377badcdab4694870f3e0054d"
        },
        "date": 1691453963319,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3283229,
            "range": "± 192309",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 288046,
            "range": "± 21962",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 106279,
            "range": "± 8165",
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
          "id": "2726d80280ab83f397ae72c0266bd91f649a6807",
          "message": "hoist declarations in nested bodies",
          "timestamp": "2023-08-08T02:28:02+02:00",
          "tree_id": "9bd055a00a4f703e8ecb39ad486101e351b160e4",
          "url": "https://github.com/y21/dash/commit/2726d80280ab83f397ae72c0266bd91f649a6807"
        },
        "date": 1691454682986,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3202772,
            "range": "± 17975",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 238002,
            "range": "± 902",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81940,
            "range": "± 376",
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
          "id": "772ff2aa95b3534efb010f5e9f5938aecfb31cff",
          "message": "implement object and array spread syntax",
          "timestamp": "2023-08-08T16:49:49+02:00",
          "tree_id": "f48f09b278fc6716e8bdcc63669b35d7c02e9e38",
          "url": "https://github.com/y21/dash/commit/772ff2aa95b3534efb010f5e9f5938aecfb31cff"
        },
        "date": 1691506408729,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3058624,
            "range": "± 8426",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 242030,
            "range": "± 2426",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 80289,
            "range": "± 513",
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
          "id": "6ff53bb90678f8670e28db25705e89ea1d899f0a",
          "message": "set `this` in get accessors when reading property (fixes #71)",
          "timestamp": "2023-08-08T21:35:51+02:00",
          "tree_id": "cb0da499d88a6549cbb0216f5ab955d038089b95",
          "url": "https://github.com/y21/dash/commit/6ff53bb90678f8670e28db25705e89ea1d899f0a"
        },
        "date": 1691523602101,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3371749,
            "range": "± 208213",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 292952,
            "range": "± 16854",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 107603,
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
          "id": "efd6ce137b407f6bf607e2e2db5c4937b85333e7",
          "message": "implement `Object.prototype.toString` in terms of its constructor",
          "timestamp": "2023-08-08T23:02:57+02:00",
          "tree_id": "30423abec3eee6c22769768318b4e39bcc8c21c0",
          "url": "https://github.com/y21/dash/commit/efd6ce137b407f6bf607e2e2db5c4937b85333e7"
        },
        "date": 1691528899786,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2882328,
            "range": "± 8756",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 242670,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 79743,
            "range": "± 83",
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
          "id": "9ab6e2cc29cc9b136a885408df8d1d293512931a",
          "message": "set the prototype and constructor of boxed primitives",
          "timestamp": "2023-08-08T23:38:24+02:00",
          "tree_id": "076f6f900bf1da3fef2d41d853aee4144058c71c",
          "url": "https://github.com/y21/dash/commit/9ab6e2cc29cc9b136a885408df8d1d293512931a"
        },
        "date": 1691530917307,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3072536,
            "range": "± 44868",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 241499,
            "range": "± 1848",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81797,
            "range": "± 364",
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
          "id": "dde2fe7921dc96f37e6614a1aad73895f577b0ee",
          "message": "add `Object.{entries,assign}`",
          "timestamp": "2023-08-09T02:18:36+02:00",
          "tree_id": "01b83132c03d6d644cccb2069d794bc088bfab6a",
          "url": "https://github.com/y21/dash/commit/dde2fe7921dc96f37e6614a1aad73895f577b0ee"
        },
        "date": 1691540558168,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3632809,
            "range": "± 150085",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 280213,
            "range": "± 4329",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 94801,
            "range": "± 207",
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
          "id": "4ae51dcab5d40c8c79599d9704b6592c925ec8c2",
          "message": "implement spread operator in argument position",
          "timestamp": "2023-08-10T03:11:18+02:00",
          "tree_id": "22acef37023482852e315018000e173ad4572c98",
          "url": "https://github.com/y21/dash/commit/4ae51dcab5d40c8c79599d9704b6592c925ec8c2"
        },
        "date": 1691630166747,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3891426,
            "range": "± 170010",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 341189,
            "range": "± 17093",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 117001,
            "range": "± 4638",
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
          "id": "1097ab1b2bbbb48bb9ff43893361cb53e0b64ca2",
          "message": "fix some new edge cases",
          "timestamp": "2023-08-11T23:13:47+02:00",
          "tree_id": "d03ada12f4a4cd611da6f8bcf26ec933fb00b432",
          "url": "https://github.com/y21/dash/commit/1097ab1b2bbbb48bb9ff43893361cb53e0b64ca2"
        },
        "date": 1691788631114,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2886206,
            "range": "± 22561",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 248828,
            "range": "± 1243",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88085,
            "range": "± 563",
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
          "id": "7e90c7edf3e54a0a4b15443ac2a81aa41db47da9",
          "message": "reorganize & redo errors",
          "timestamp": "2023-08-12T23:19:55+02:00",
          "tree_id": "17c784c3c85e466c6a6bb352060431df3dce2405",
          "url": "https://github.com/y21/dash/commit/7e90c7edf3e54a0a4b15443ac2a81aa41db47da9"
        },
        "date": 1691875439294,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3418209,
            "range": "± 5410",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 297903,
            "range": "± 938",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 103846,
            "range": "± 319",
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
          "id": "e94995100d7269fde1a346917e79cf4b10ae6749",
          "message": "fix buggy class member lowering",
          "timestamp": "2023-08-13T05:04:18+02:00",
          "tree_id": "3d4fea0fbb1b72e5fa9b5f86028558aa2d7ffe73",
          "url": "https://github.com/y21/dash/commit/e94995100d7269fde1a346917e79cf4b10ae6749"
        },
        "date": 1691896067352,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2901753,
            "range": "± 4867",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 249874,
            "range": "± 2127",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88423,
            "range": "± 294",
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
          "id": "182fd2e3812993c90cdb3115de2d0429d8a00e98",
          "message": "add spans to compiler errors",
          "timestamp": "2023-08-13T22:26:25+02:00",
          "tree_id": "20fb54d9a01de2f930e258635ff218e2eed84c83",
          "url": "https://github.com/y21/dash/commit/182fd2e3812993c90cdb3115de2d0429d8a00e98"
        },
        "date": 1691958604167,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2932324,
            "range": "± 12147",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 252931,
            "range": "± 835",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88928,
            "range": "± 244",
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
          "id": "182fd2e3812993c90cdb3115de2d0429d8a00e98",
          "message": "Rework the AST",
          "timestamp": "2023-04-14T21:40:49Z",
          "url": "https://github.com/y21/dash/pull/72/commits/182fd2e3812993c90cdb3115de2d0429d8a00e98"
        },
        "date": 1691958856315,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2898616,
            "range": "± 25046",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 252258,
            "range": "± 1520",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88610,
            "range": "± 361",
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
          "id": "51e735a9e48226e06e33f4a4e0935478f8bf51d7",
          "message": "Merge pull request #72 from y21/spans\n\nRework the AST",
          "timestamp": "2023-08-13T22:34:49+02:00",
          "tree_id": "20fb54d9a01de2f930e258635ff218e2eed84c83",
          "url": "https://github.com/y21/dash/commit/51e735a9e48226e06e33f4a4e0935478f8bf51d7"
        },
        "date": 1691959136519,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3873033,
            "range": "± 147175",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 342095,
            "range": "± 21445",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 120990,
            "range": "± 8111",
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
          "id": "3e858214acddeaf15d86c5d8982125d0710cb8a4",
          "message": "precompute static property access hashes",
          "timestamp": "2023-08-16T21:38:07+02:00",
          "tree_id": "13770dfaae0244d5185343e87432e7f939ba1bd8",
          "url": "https://github.com/y21/dash/commit/3e858214acddeaf15d86c5d8982125d0710cb8a4"
        },
        "date": 1692214906350,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2821864,
            "range": "± 17675",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 243248,
            "range": "± 1771",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 81186,
            "range": "± 1724",
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
          "id": "5604eeeaaa87869a2f8729eea31dfeebe21cb371",
          "message": "avoid rehash/re-rc when interning new entry",
          "timestamp": "2023-08-25T13:21:51+02:00",
          "tree_id": "0438f4503ccd42a8a4b4c306319346ca638e47d3",
          "url": "https://github.com/y21/dash/commit/5604eeeaaa87869a2f8729eea31dfeebe21cb371"
        },
        "date": 1692962721474,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3113160,
            "range": "± 8106",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 252692,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 87591,
            "range": "± 585",
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
          "id": "9f9cb56ed031751bd02497ccf238ae32f9d9a1dd",
          "message": "initial type checker",
          "timestamp": "2023-08-25T19:10:57+02:00",
          "tree_id": "f20881b780fb6a3681e47b362f66f088681de584",
          "url": "https://github.com/y21/dash/commit/9f9cb56ed031751bd02497ccf238ae32f9d9a1dd"
        },
        "date": 1692983666077,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3128199,
            "range": "± 101151",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 251828,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88580,
            "range": "± 425",
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
          "id": "c954aa5caee00960a62d11b86baee12665752181",
          "message": "make it compile",
          "timestamp": "2023-08-25T19:12:54+02:00",
          "tree_id": "525f2f3139c40252e2c056a7102b47fab8ca2f28",
          "url": "https://github.com/y21/dash/commit/c954aa5caee00960a62d11b86baee12665752181"
        },
        "date": 1692983797673,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2963625,
            "range": "± 10634",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 257090,
            "range": "± 2995",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88777,
            "range": "± 571",
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
          "id": "0ef3f6f62732cd55ebeb1327b6689396056f30a1",
          "message": "unambiguously parse `get` property keys",
          "timestamp": "2023-08-26T18:20:21+02:00",
          "tree_id": "f56fdb75e83511abcfc9b842367a10b118e4ff4e",
          "url": "https://github.com/y21/dash/commit/0ef3f6f62732cd55ebeb1327b6689396056f30a1"
        },
        "date": 1693067030704,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3123979,
            "range": "± 7392",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 254506,
            "range": "± 1472",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 87693,
            "range": "± 355",
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
          "id": "c067f4c24a8872dbf81a8acd8ce5c2142d0dc7ff",
          "message": "implement `Object` constructor",
          "timestamp": "2023-08-26T20:43:30+02:00",
          "tree_id": "edce7ee69e203d69d87eb7a18a438bfdd6533f10",
          "url": "https://github.com/y21/dash/commit/c067f4c24a8872dbf81a8acd8ce5c2142d0dc7ff"
        },
        "date": 1693075629287,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2954212,
            "range": "± 9266",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 256806,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 88145,
            "range": "± 600",
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
          "id": "c82d40985cac538b6537ba57bfbdde59bb33a23b",
          "message": "don't call ToNumber in Number.prototype.toString",
          "timestamp": "2023-08-26T21:15:25+02:00",
          "tree_id": "0c1b7ae30f3a7ee5796828e9caac1d3ba4ac5c23",
          "url": "https://github.com/y21/dash/commit/c82d40985cac538b6537ba57bfbdde59bb33a23b"
        },
        "date": 1693077628105,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 4230539,
            "range": "± 312942",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 376944,
            "range": "± 19099",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 141677,
            "range": "± 10692",
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
          "id": "eb34c258ced3f4b85241181df9e7025c287ffa29",
          "message": "allow trailing comma in obj dstr, lex hex escape sequence",
          "timestamp": "2023-08-26T22:07:10+02:00",
          "tree_id": "149ffde3163e1ec18ca714304779d444d62de2d5",
          "url": "https://github.com/y21/dash/commit/eb34c258ced3f4b85241181df9e7025c287ffa29"
        },
        "date": 1693080704662,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3678444,
            "range": "± 233202",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 322011,
            "range": "± 33235",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 118268,
            "range": "± 5593",
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
          "id": "0441adcdd35b30edc6a81fd255663e2d787803b2",
          "message": "remove unnecessary `RefCell`s",
          "timestamp": "2023-10-09T00:24:38+02:00",
          "tree_id": "4d307f086bfd973e6c15bd0ef1c458492ceef2cb",
          "url": "https://github.com/y21/dash/commit/0441adcdd35b30edc6a81fd255663e2d787803b2"
        },
        "date": 1696804096418,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2781768,
            "range": "± 32323",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 233955,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 85586,
            "range": "± 360",
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
          "id": "d5613040ed434b95614e668c94ebe2b8f5c27e4e",
          "message": "don't attempt to parse import specifier as ident",
          "timestamp": "2023-10-09T17:41:14+02:00",
          "tree_id": "9a513efbd7b4526283727496e924e6e5ae9d8c7c",
          "url": "https://github.com/y21/dash/commit/d5613040ed434b95614e668c94ebe2b8f5c27e4e"
        },
        "date": 1696866295027,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2803945,
            "range": "± 9234",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 232695,
            "range": "± 1738",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 84257,
            "range": "± 485",
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
          "id": "abcf758c7aa657367b98605bc9eb40485a472fdf",
          "message": "refactor intrinsic register macro to fn",
          "timestamp": "2023-10-11T23:48:23+02:00",
          "tree_id": "5ee4fa7c3caebbb374125202af3015ebb5ac202b",
          "url": "https://github.com/y21/dash/commit/abcf758c7aa657367b98605bc9eb40485a472fdf"
        },
        "date": 1697061123642,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2789171,
            "range": "± 10726",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 237159,
            "range": "± 2053",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83610,
            "range": "± 256",
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
          "id": "1325fcea915c538508c0fa8ab48514ff2feebfc7",
          "message": "wip",
          "timestamp": "2023-10-13T17:24:54+02:00",
          "tree_id": "77c7d5809ade885f34bdaf993ec3b89b2f06f74a",
          "url": "https://github.com/y21/dash/commit/1325fcea915c538508c0fa8ab48514ff2feebfc7"
        },
        "date": 1697210941003,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2810330,
            "range": "± 9711",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 230405,
            "range": "± 813",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83623,
            "range": "± 271",
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
          "id": "596b3fb749a5b238670840165304659c831278e6",
          "message": "fix incorrect suberror prototypes & unused warnings",
          "timestamp": "2023-10-15T19:50:55+02:00",
          "tree_id": "72552180f22256da35ef2593596da0e9a5145967",
          "url": "https://github.com/y21/dash/commit/596b3fb749a5b238670840165304659c831278e6"
        },
        "date": 1697392473204,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2786763,
            "range": "± 6581",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 230154,
            "range": "± 1076",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82995,
            "range": "± 384",
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
          "id": "a04f277a4429f9b14dc8108334a92c50966a7b00",
          "message": "support recursive require",
          "timestamp": "2023-10-20T19:40:43+02:00",
          "tree_id": "7e771e911a2ea7c8332758f187064b09b3315ab4",
          "url": "https://github.com/y21/dash/commit/a04f277a4429f9b14dc8108334a92c50966a7b00"
        },
        "date": 1697823895214,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3472799,
            "range": "± 107762",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 295956,
            "range": "± 15349",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 117455,
            "range": "± 4430",
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
          "id": "c613d7441f3e8f45f938a86b5c4435bc0e7afbf5",
          "message": "use resolver 2",
          "timestamp": "2023-10-20T21:39:13+02:00",
          "tree_id": "0d290da0b0e18b02236fdf98c2b41342ff3a47ad",
          "url": "https://github.com/y21/dash/commit/c613d7441f3e8f45f938a86b5c4435bc0e7afbf5"
        },
        "date": 1697830955036,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2916285,
            "range": "± 7169",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 232937,
            "range": "± 2081",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82182,
            "range": "± 272",
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
          "id": "cd6a1f87ea8f60669fd51b8b5f06bc1aa4102785",
          "message": "make `require` local to modules instead of globals",
          "timestamp": "2023-10-22T01:48:08+02:00",
          "tree_id": "73b97dc2627f3ea01dba9bac26087a9214d1b1c2",
          "url": "https://github.com/y21/dash/commit/cd6a1f87ea8f60669fd51b8b5f06bc1aa4102785"
        },
        "date": 1697932292687,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2790682,
            "range": "± 5723",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 232805,
            "range": "± 827",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82955,
            "range": "± 712",
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
          "id": "cd6a1f87ea8f60669fd51b8b5f06bc1aa4102785",
          "message": "initial node-compat mode",
          "timestamp": "2023-04-14T21:40:49Z",
          "url": "https://github.com/y21/dash/pull/73/commits/cd6a1f87ea8f60669fd51b8b5f06bc1aa4102785"
        },
        "date": 1697932689778,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2826501,
            "range": "± 5228",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 242351,
            "range": "± 1768",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83647,
            "range": "± 1095",
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
          "id": "c90d51549a104ea990a04d7358c335eceef8ff01",
          "message": "Merge pull request #73 from y21/node\n\ninitial node-compat mode",
          "timestamp": "2023-10-22T02:02:16+02:00",
          "tree_id": "b2285ec2916dc2f4513313b3dd22af90b15b22cb",
          "url": "https://github.com/y21/dash/commit/c90d51549a104ea990a04d7358c335eceef8ff01"
        },
        "date": 1697933140856,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2782892,
            "range": "± 3341",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 236446,
            "range": "± 1533",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83138,
            "range": "± 295",
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
          "id": "5fa4d8d97ee62bea58ef6ef2f25ec6b1375f6d09",
          "message": "fix ternary operator precedence being above assignment",
          "timestamp": "2023-10-22T02:45:34+02:00",
          "tree_id": "b3244f4005e5c36076bf8f31fee6d9e8287804cc",
          "url": "https://github.com/y21/dash/commit/5fa4d8d97ee62bea58ef6ef2f25ec6b1375f6d09"
        },
        "date": 1697935758326,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 3225954,
            "range": "± 64430",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 268083,
            "range": "± 6708",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 97148,
            "range": "± 1639",
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
          "id": "a5be72866937b2e452be035b1c173dbe8d359dcd",
          "message": "resolve sequence parsing ambiguity",
          "timestamp": "2023-10-22T03:59:22+02:00",
          "tree_id": "b13073a284420320a517224a40ba5e6f9ec0a149",
          "url": "https://github.com/y21/dash/commit/a5be72866937b2e452be035b1c173dbe8d359dcd"
        },
        "date": 1697940171734,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2778721,
            "range": "± 3953",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 231377,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 83457,
            "range": "± 342",
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
          "id": "441c91fcab824d76a8c318efdb6bb633dd280c1a",
          "message": "regex: support character class ranges",
          "timestamp": "2023-10-23T00:01:57+02:00",
          "tree_id": "2bcb26ab834e63325cbfca09cfb82bf1a8856024",
          "url": "https://github.com/y21/dash/commit/441c91fcab824d76a8c318efdb6bb633dd280c1a"
        },
        "date": 1698012329552,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2874575,
            "range": "± 18412",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 236144,
            "range": "± 1585",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 82756,
            "range": "± 228",
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
          "id": "c461504c92c4fe747967bc4246abb4d577e1b4a8",
          "message": "support rest parameters in arrow functions",
          "timestamp": "2023-12-23T01:30:42+01:00",
          "tree_id": "9723350151ee17ad86d803d0dd2688c8aa13efc9",
          "url": "https://github.com/y21/dash/commit/c461504c92c4fe747967bc4246abb4d577e1b4a8"
        },
        "date": 1703291594147,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2400120,
            "range": "± 17062",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 196626,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57311,
            "range": "± 1227",
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
          "id": "afd8c9c099447daea2bd334ccd3914659378d7a8",
          "message": "reformat code base and add stress_gc feature",
          "timestamp": "2023-12-23T22:12:30+01:00",
          "tree_id": "f60c0d1f98d7bea6f77b700d58827ffba5809a03",
          "url": "https://github.com/y21/dash/commit/afd8c9c099447daea2bd334ccd3914659378d7a8"
        },
        "date": 1703366115949,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2517312,
            "range": "± 6552",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 228425,
            "range": "± 951",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58679,
            "range": "± 2279",
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
          "id": "e4a83bf7c497ad5a90ea8d3a4bb434371f52af7e",
          "message": "move `ValueConversions` to common mod, commit rustfmt.toml",
          "timestamp": "2023-12-24T00:58:45+01:00",
          "tree_id": "33b8367e2ad172aa7d1447cae36564659be16f63",
          "url": "https://github.com/y21/dash/commit/e4a83bf7c497ad5a90ea8d3a4bb434371f52af7e"
        },
        "date": 1703376068703,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2527513,
            "range": "± 37067",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 228696,
            "range": "± 2358",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59320,
            "range": "± 1331",
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
          "id": "4cd9cfdc2f174ecab86c42b5f5369670976b09d2",
          "message": "show callee source code in \"is not a function\" errors\n\nFixes #75",
          "timestamp": "2023-12-24T15:14:48+01:00",
          "tree_id": "12d719900ba8af8df1e86744415b5fcf68e3ee11",
          "url": "https://github.com/y21/dash/commit/4cd9cfdc2f174ecab86c42b5f5369670976b09d2"
        },
        "date": 1703427436562,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2564527,
            "range": "± 14352",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 234511,
            "range": "± 3745",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58750,
            "range": "± 339",
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
          "id": "e02aa62ca3092d091b0a8b84ec6c31f27187c6c7",
          "message": "retry regex matching when first match fails",
          "timestamp": "2023-12-24T18:58:24+01:00",
          "tree_id": "a317366d33e8b5c2f7f00d462f78f6f083d24aa3",
          "url": "https://github.com/y21/dash/commit/e02aa62ca3092d091b0a8b84ec6c31f27187c6c7"
        },
        "date": 1703440851677,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2566877,
            "range": "± 13831",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 230208,
            "range": "± 3487",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58108,
            "range": "± 370",
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
          "id": "f6d7b42316b3ad18b5413d27c6f02f7fdcf6146d",
          "message": "implement `lastIndex` regex logic",
          "timestamp": "2023-12-24T20:26:45+01:00",
          "tree_id": "4ea1882eba72dff7faf493ee6ab56ca2d5842dd4",
          "url": "https://github.com/y21/dash/commit/f6d7b42316b3ad18b5413d27c6f02f7fdcf6146d"
        },
        "date": 1703446160683,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2580256,
            "range": "± 54102",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 242634,
            "range": "± 1894",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59765,
            "range": "± 676",
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
          "id": "4ae8aa11628f2091b1bc8d2caa0a78ceaa94b140",
          "message": "implement non-capturing regex groups and properly reset `lastIndex` to 0",
          "timestamp": "2023-12-24T23:00:39+01:00",
          "tree_id": "fe6299da4e75b00f3d5754220581a8a495b401b2",
          "url": "https://github.com/y21/dash/commit/4ae8aa11628f2091b1bc8d2caa0a78ceaa94b140"
        },
        "date": 1703455400240,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2549561,
            "range": "± 30803",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 230497,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57233,
            "range": "± 687",
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
          "id": "9cfb26145e1aa0d6c6c7369559e718d16906d650",
          "message": "add `RegExp.prototype.exec`",
          "timestamp": "2023-12-25T01:16:09+01:00",
          "tree_id": "d1c5806d53eb288f94ff948ef273f994a40119d7",
          "url": "https://github.com/y21/dash/commit/9cfb26145e1aa0d6c6c7369559e718d16906d650"
        },
        "date": 1703463524778,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2598651,
            "range": "± 34048",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 234916,
            "range": "± 1293",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58829,
            "range": "± 2647",
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
          "id": "8e092ad24c374963678e5d98f18edc5d82ab141a",
          "message": "delay regex flag parsing",
          "timestamp": "2023-12-25T20:56:38+01:00",
          "tree_id": "a9ff307dfe2854af3798eb112f7dcced305a4fca",
          "url": "https://github.com/y21/dash/commit/8e092ad24c374963678e5d98f18edc5d82ab141a"
        },
        "date": 1703534343078,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2583337,
            "range": "± 7671",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 251987,
            "range": "± 2340",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59526,
            "range": "± 335",
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
          "id": "b71cd5ce91cb120c14d8626e0dee04c61234c5a6",
          "message": "specialize `intern_char`",
          "timestamp": "2023-12-27T06:35:00+01:00",
          "tree_id": "662a8e6d1813e2f4cd292a0a71bc449f348bbea0",
          "url": "https://github.com/y21/dash/commit/b71cd5ce91cb120c14d8626e0dee04c61234c5a6"
        },
        "date": 1703655448794,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1866926,
            "range": "± 25608",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174051,
            "range": "± 1032",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55041,
            "range": "± 2020",
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
          "id": "b71cd5ce91cb120c14d8626e0dee04c61234c5a6",
          "message": "dynamically intern strings at runtime",
          "timestamp": "2023-12-13T19:08:01Z",
          "url": "https://github.com/y21/dash/pull/76/commits/b71cd5ce91cb120c14d8626e0dee04c61234c5a6"
        },
        "date": 1703656215411,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1888041,
            "range": "± 37236",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 173043,
            "range": "± 1155",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55084,
            "range": "± 1212",
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
          "id": "5f16becff01a9f8e71ccd0302249ce78f105a02c",
          "message": "Merge pull request #76 from y21/intern-strings-vm-2\n\ndynamically intern strings at runtime",
          "timestamp": "2023-12-27T06:56:32+01:00",
          "tree_id": "662a8e6d1813e2f4cd292a0a71bc449f348bbea0",
          "url": "https://github.com/y21/dash/commit/5f16becff01a9f8e71ccd0302249ce78f105a02c"
        },
        "date": 1703656742645,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1885939,
            "range": "± 12498",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 175098,
            "range": "± 1470",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55075,
            "range": "± 2993",
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
          "id": "f61eeaa03dca0f115d88e46ee68ed5e6c836c879",
          "message": "add `Object.prototype.isPrototypeOf` and `Object.getPrototypeOf`",
          "timestamp": "2023-12-27T16:36:28+01:00",
          "tree_id": "170065d696fcfaa0cbe94516d780ec2d6f7689bf",
          "url": "https://github.com/y21/dash/commit/f61eeaa03dca0f115d88e46ee68ed5e6c836c879"
        },
        "date": 1703691539222,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1912725,
            "range": "± 18176",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 179050,
            "range": "± 1862",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55751,
            "range": "± 3840",
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
          "id": "cf2954d9a65d12b6dccdc05418874ce32cbcd605",
          "message": "avoid tracing preinterned symbols and instead don't check them during sweep",
          "timestamp": "2023-12-27T20:12:23+01:00",
          "tree_id": "cdffb20f2da36b9122d82709a1c9c290c39e37aa",
          "url": "https://github.com/y21/dash/commit/cf2954d9a65d12b6dccdc05418874ce32cbcd605"
        },
        "date": 1703704491690,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1895333,
            "range": "± 14491",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 175183,
            "range": "± 1155",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55510,
            "range": "± 5015",
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
          "id": "54678f21f08e6680f0d773cd515bc5881e2d1d3c",
          "message": "disassociate `new` exprs in function call arguments",
          "timestamp": "2023-12-27T21:24:18+01:00",
          "tree_id": "c03e15bcf9e59f0a049073bc284d44b1d75c9de9",
          "url": "https://github.com/y21/dash/commit/54678f21f08e6680f0d773cd515bc5881e2d1d3c"
        },
        "date": 1703708810271,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1876259,
            "range": "± 22847",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171379,
            "range": "± 506",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55760,
            "range": "± 505",
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
          "id": "cdf41f828542995c310384c58f92418c96b36a71",
          "message": "pop class values correctly in class declaration statements",
          "timestamp": "2023-12-27T22:57:14+01:00",
          "tree_id": "e26ba0a0c1575fbcd4414cac5a4235a30b73a18a",
          "url": "https://github.com/y21/dash/commit/cdf41f828542995c310384c58f92418c96b36a71"
        },
        "date": 1703714385386,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1887750,
            "range": "± 30918",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 180685,
            "range": "± 3192",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57472,
            "range": "± 2041",
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
          "id": "487273cfb88f135494e4c90a7a3904e7dec2e610",
          "message": "use fxhash for object maps\n\nall strings are interned now and are represented using `u32`s, so `FxHash` works well here",
          "timestamp": "2023-12-28T03:14:49+01:00",
          "tree_id": "7551c4f7d083887dde5b4da9fc115661f9ba689b",
          "url": "https://github.com/y21/dash/commit/487273cfb88f135494e4c90a7a3904e7dec2e610"
        },
        "date": 1703729844273,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1849651,
            "range": "± 50994",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 163615,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 51926,
            "range": "± 2526",
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
          "id": "63013223e6b11a2a2b233fc336a6d02cae6c6233",
          "message": "specialize `usize` interning",
          "timestamp": "2023-12-28T04:30:17+01:00",
          "tree_id": "3694438901ce918914e2de661850adc5c065c7f9",
          "url": "https://github.com/y21/dash/commit/63013223e6b11a2a2b233fc336a6d02cae6c6233"
        },
        "date": 1703734365117,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1831377,
            "range": "± 81106",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 162122,
            "range": "± 704",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 52906,
            "range": "± 1503",
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
          "id": "23a5583478c1afe1a3c042c092dc29c685ffcc34",
          "message": "properly display string and idents in IR dump",
          "timestamp": "2023-12-28T17:45:08+01:00",
          "tree_id": "9ec776b0e4e84674e57efb1ba0d80b521f66af10",
          "url": "https://github.com/y21/dash/commit/23a5583478c1afe1a3c042c092dc29c685ffcc34"
        },
        "date": 1703782064329,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1806646,
            "range": "± 15217",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 160732,
            "range": "± 4534",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 52940,
            "range": "± 2916",
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
          "id": "b008b8ac08d3cdf0d1e08c1d8185ef69c1806a93",
          "message": "fix sequence precedence test and add tests for multi variable declaration",
          "timestamp": "2023-12-28T18:01:43+01:00",
          "tree_id": "5ed713eddd0b205dbb2995bf454f29a4d2e0a22b",
          "url": "https://github.com/y21/dash/commit/b008b8ac08d3cdf0d1e08c1d8185ef69c1806a93"
        },
        "date": 1703783054127,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1780533,
            "range": "± 13641",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 161913,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 52193,
            "range": "± 1685",
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
          "id": "0144323108d35e28d54a713279e4ff18c5cf1c8c",
          "message": "simplify implementation of external values",
          "timestamp": "2023-12-29T01:55:20+01:00",
          "tree_id": "8e33d24d54b6b83bebb7028506b55b08dc296f03",
          "url": "https://github.com/y21/dash/commit/0144323108d35e28d54a713279e4ff18c5cf1c8c"
        },
        "date": 1703811471371,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1853874,
            "range": "± 51511",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171255,
            "range": "± 2245",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53661,
            "range": "± 2992",
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
          "id": "46445aca1e067694c90a7b2ea81caa5ec42b8cd9",
          "message": "make `Handle` and `Persistent` the size of a machine pointer",
          "timestamp": "2023-12-29T18:35:10+01:00",
          "tree_id": "8bbbb7d13bc6a577d0d66686b33d9b1b817e2223",
          "url": "https://github.com/y21/dash/commit/46445aca1e067694c90a7b2ea81caa5ec42b8cd9"
        },
        "date": 1703871460167,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1747551,
            "range": "± 19443",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 175373,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54623,
            "range": "± 1929",
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
          "id": "e891408e120b01aa0440e5736c624cce6d2918d5",
          "message": "avoid unnecessary stack push/pop in specialized `iltconst32`",
          "timestamp": "2023-12-29T19:11:42+01:00",
          "tree_id": "82d24a7330eb8e6d38cbbacbcd75227d02c36229",
          "url": "https://github.com/y21/dash/commit/e891408e120b01aa0440e5736c624cce6d2918d5"
        },
        "date": 1703873652854,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1749434,
            "range": "± 18959",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168940,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53347,
            "range": "± 1117",
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
          "id": "80775aa6e444408b45bc9f9101aa954844863acb",
          "message": "gracefully handle unknown instructions",
          "timestamp": "2023-10-15T21:22:34+02:00",
          "tree_id": "d1f7bff71f6245a9c36c7621d6e581d4bfc40c5c",
          "url": "https://github.com/y21/dash/commit/80775aa6e444408b45bc9f9101aa954844863acb"
        },
        "date": 1703882236586,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 2327934,
            "range": "± 111831",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 188632,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58868,
            "range": "± 275",
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
          "id": "67c98648ed71dc54beed0eef1fbf91217d5215a1",
          "message": "extract escape character parsing",
          "timestamp": "2023-12-30T00:05:01+01:00",
          "tree_id": "10ed20dd0c258dbf39f31492b3c179078bcc5ce8",
          "url": "https://github.com/y21/dash/commit/67c98648ed71dc54beed0eef1fbf91217d5215a1"
        },
        "date": 1703891250581,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1738872,
            "range": "± 20811",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 172560,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53483,
            "range": "± 2685",
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
          "id": "2e1f3367bad5c5642542268050182d8295f9eac8",
          "message": "fix strict equality and primitive constructors",
          "timestamp": "2024-01-15T00:35:43+01:00",
          "tree_id": "716f5a241011d473a88fd951b795f330bb44eb1f",
          "url": "https://github.com/y21/dash/commit/2e1f3367bad5c5642542268050182d8295f9eac8"
        },
        "date": 1705275494842,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1753144,
            "range": "± 15708",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168600,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53739,
            "range": "± 244",
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
          "id": "1a0bf22bc3662e455ef9746c0cf787f7a9e4d18a",
          "message": "box regex constants",
          "timestamp": "2024-01-15T01:41:44+01:00",
          "tree_id": "00655fe3594a87d19a2546e50a22b043d0fa91dc",
          "url": "https://github.com/y21/dash/commit/1a0bf22bc3662e455ef9746c0cf787f7a9e4d18a"
        },
        "date": 1705279453287,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1771899,
            "range": "± 14616",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174223,
            "range": "± 427",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53979,
            "range": "± 1774",
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
          "id": "08928db7f3dbee2de970ad75cc79ec3ae42399f3",
          "message": "lower `switch` using if-else instead of its own opcode\n\nthe previous lowering was wrong, this fixes it",
          "timestamp": "2024-01-16T19:59:33+01:00",
          "tree_id": "9db17d1323baa57b32f2f782a8983f4cbcac0283",
          "url": "https://github.com/y21/dash/commit/08928db7f3dbee2de970ad75cc79ec3ae42399f3"
        },
        "date": 1705431723365,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1747951,
            "range": "± 25896",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 165822,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53978,
            "range": "± 1342",
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
          "id": "9a585ca8bdf82b21a66b4cf725248050fed51035",
          "message": "special case empty separators in string split",
          "timestamp": "2024-01-17T21:10:51+01:00",
          "tree_id": "5d57c0ea7be858e2c0f84a073d116bb8a166115a",
          "url": "https://github.com/y21/dash/commit/9a585ca8bdf82b21a66b4cf725248050fed51035"
        },
        "date": 1705522412145,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1738120,
            "range": "± 12249",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 167927,
            "range": "± 721",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54817,
            "range": "± 2212",
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
          "id": "a51af38c57bc2c8957b078c8d47d6fba87f18f0c",
          "message": "parse multiple variable declarations in for loop correctly",
          "timestamp": "2024-01-17T22:31:16+01:00",
          "tree_id": "70b0a1e456137c79a9bdfd8a44d1b6e6ffdda205",
          "url": "https://github.com/y21/dash/commit/a51af38c57bc2c8957b078c8d47d6fba87f18f0c"
        },
        "date": 1705527226425,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1730612,
            "range": "± 86301",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171884,
            "range": "± 1231",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54777,
            "range": "± 280",
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
          "id": "50ffedffa0953567fb3b1957aea03246ca057e6d",
          "message": "node: return `export` property for cached modules",
          "timestamp": "2024-01-17T22:58:58+01:00",
          "tree_id": "de66d56c1d15772b7fed576910565a0c5a501c43",
          "url": "https://github.com/y21/dash/commit/50ffedffa0953567fb3b1957aea03246ca057e6d"
        },
        "date": 1705528898899,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1719277,
            "range": "± 20232",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171332,
            "range": "± 1096",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54560,
            "range": "± 1057",
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
          "id": "b5e9f3c104384cebaffd05413bbe4b6129a4e7a5",
          "message": "add globally-scoped eval",
          "timestamp": "2024-01-18T21:01:56+01:00",
          "tree_id": "34b8d412081fcea8e02a49c6cdb65c05722e941f",
          "url": "https://github.com/y21/dash/commit/b5e9f3c104384cebaffd05413bbe4b6129a4e7a5"
        },
        "date": 1705608273826,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1719215,
            "range": "± 17156",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 169758,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53480,
            "range": "± 246",
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
          "id": "50e4b50cf02366c93d2c79fcdf0c21db8d37fba2",
          "message": "clippy",
          "timestamp": "2024-01-18T23:41:44+01:00",
          "tree_id": "fe2443b8e3e199cd75b9426c5163b313213c7ad8",
          "url": "https://github.com/y21/dash/commit/50e4b50cf02366c93d2c79fcdf0c21db8d37fba2"
        },
        "date": 1705617867123,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1723808,
            "range": "± 13226",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 162804,
            "range": "± 559",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55786,
            "range": "± 864",
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
          "id": "fbbebb2b1cac119d3ad36c522a2016e1e70d5061",
          "message": "node: support implicit js extensions in path resolution",
          "timestamp": "2024-01-19T00:20:53+01:00",
          "tree_id": "839f5bb116e2804ecd305ce22ec99a8e059b27cc",
          "url": "https://github.com/y21/dash/commit/fbbebb2b1cac119d3ad36c522a2016e1e70d5061"
        },
        "date": 1705620207873,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1730846,
            "range": "± 11437",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 164105,
            "range": "± 1182",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55060,
            "range": "± 2010",
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
          "id": "6dd9ebf63daca0e83dee0d1b8e59a4d06213a6cd",
          "message": "allow direct assignments to `function.prototype`",
          "timestamp": "2024-01-19T00:42:04+01:00",
          "tree_id": "397724cd3a08c4500222d0438e502593ba9172c9",
          "url": "https://github.com/y21/dash/commit/6dd9ebf63daca0e83dee0d1b8e59a4d06213a6cd"
        },
        "date": 1705621470327,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1748315,
            "range": "± 52673",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168245,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 56113,
            "range": "± 303",
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
          "id": "4432293ebe14d30fccc5b5fed681deb6d8664511",
          "message": "replace `ValueEquality` with free fns",
          "timestamp": "2024-02-16T17:59:49+01:00",
          "tree_id": "2a441231dadaec24f53345e1b438adc58b130a73",
          "url": "https://github.com/y21/dash/commit/4432293ebe14d30fccc5b5fed681deb6d8664511"
        },
        "date": 1708102934122,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1666876,
            "range": "± 25284",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168511,
            "range": "± 2384",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55417,
            "range": "± 285",
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
          "id": "7748b453a0ee3b8dcdcca75a91bbc1ce3ea633cc",
          "message": "respect non-writable property descriptors when setting properties",
          "timestamp": "2024-02-17T10:12:22+01:00",
          "tree_id": "912171965a9d13b74b5111b4a0f7685cd7b151c7",
          "url": "https://github.com/y21/dash/commit/7748b453a0ee3b8dcdcca75a91bbc1ce3ea633cc"
        },
        "date": 1708161291149,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1675720,
            "range": "± 19511",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 165022,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 53696,
            "range": "± 1026",
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
          "id": "056f0fec5d03c416adeedb600dc44d93a9f05234",
          "message": "reorder instruction match to match definition order",
          "timestamp": "2024-02-17T14:12:38+01:00",
          "tree_id": "06f2fdedabcde2d4ec248995b281232108772707",
          "url": "https://github.com/y21/dash/commit/056f0fec5d03c416adeedb600dc44d93a9f05234"
        },
        "date": 1708175707556,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1665960,
            "range": "± 20745",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 167134,
            "range": "± 1733",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54206,
            "range": "± 1990",
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
          "id": "6cb02c31fbf95201438cbe658e981f9b6385c837",
          "message": "specialize interning `f64`s that fit in a usize",
          "timestamp": "2024-02-17T15:53:33+01:00",
          "tree_id": "97da984f8c352ba70311b4e082e1fcf1ca52a272",
          "url": "https://github.com/y21/dash/commit/6cb02c31fbf95201438cbe658e981f9b6385c837"
        },
        "date": 1708181754890,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1667850,
            "range": "± 16261",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 166332,
            "range": "± 2785",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54962,
            "range": "± 1923",
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
          "id": "44799f86cc5860edec5176d147ddcae34908ae55",
          "message": "implement `Object.prototype.propertyIsEnumerable`\n\nrequired by test262's propertyHelper.js",
          "timestamp": "2024-02-17T18:05:01+01:00",
          "tree_id": "9686e75c646edbf88ed7da1ba452db1ff9506a15",
          "url": "https://github.com/y21/dash/commit/44799f86cc5860edec5176d147ddcae34908ae55"
        },
        "date": 1708189646650,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1661147,
            "range": "± 89915",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168225,
            "range": "± 8582",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55386,
            "range": "± 193",
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
          "id": "e552970d3108b19ded1144aaeec5eaf7ee200db7",
          "message": "review uses of `PropertyValue::static_default`, replace with correct descriptor",
          "timestamp": "2024-02-17T18:31:52+01:00",
          "tree_id": "032b6868198014f4b58029d96134e0b65290a436",
          "url": "https://github.com/y21/dash/commit/e552970d3108b19ded1144aaeec5eaf7ee200db7"
        },
        "date": 1708191263061,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1653352,
            "range": "± 22926",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 172092,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55313,
            "range": "± 1547",
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
          "id": "9726300fa7ff409d5ae4d76065d02e72daa068bf",
          "message": "allow closure calls to be flat",
          "timestamp": "2024-03-02T21:14:38+01:00",
          "tree_id": "a0bbd65df0b6ed1ccb66a7b32fc60328b7728447",
          "url": "https://github.com/y21/dash/commit/9726300fa7ff409d5ae4d76065d02e72daa068bf"
        },
        "date": 1709410619223,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1649494,
            "range": "± 29099",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 169269,
            "range": "± 896",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55125,
            "range": "± 235",
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
          "id": "9726300fa7ff409d5ae4d76065d02e72daa068bf",
          "message": "Preserve `this` in closures",
          "timestamp": "2024-02-20T02:40:11Z",
          "url": "https://github.com/y21/dash/pull/79/commits/9726300fa7ff409d5ae4d76065d02e72daa068bf"
        },
        "date": 1709410626293,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1660183,
            "range": "± 53570",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 169290,
            "range": "± 2522",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54807,
            "range": "± 725",
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
          "id": "530e010bdacbeba81871bec334f2853bac351399",
          "message": "Merge pull request #79 from y21/closure-this\n\nPreserve `this` in closures",
          "timestamp": "2024-03-02T21:21:46+01:00",
          "tree_id": "a0bbd65df0b6ed1ccb66a7b32fc60328b7728447",
          "url": "https://github.com/y21/dash/commit/530e010bdacbeba81871bec334f2853bac351399"
        },
        "date": 1709411051159,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1651838,
            "range": "± 65318",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 165643,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55803,
            "range": "± 993",
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
          "id": "e352f58812eee69dd686b8e7b3dbf2f37d82125d",
          "message": "require object types to have an alignment of 8 for computing offsets\n\nfixes #80",
          "timestamp": "2024-03-03T14:24:19+01:00",
          "tree_id": "8f46fd39c409c2bcd2af868f3d034e494ca550ef",
          "url": "https://github.com/y21/dash/commit/e352f58812eee69dd686b8e7b3dbf2f37d82125d"
        },
        "date": 1709472414573,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1648490,
            "range": "± 78700",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168751,
            "range": "± 3065",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54267,
            "range": "± 573",
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
          "id": "2b9024403f4e989d564751464bb3ee63e1a739c3",
          "message": "implement experimental support for holey arrays",
          "timestamp": "2024-03-17T14:35:33+01:00",
          "tree_id": "4893c9320c51a5484c0c5b4ff1fcea2b91dfd1b5",
          "url": "https://github.com/y21/dash/commit/2b9024403f4e989d564751464bb3ee63e1a739c3"
        },
        "date": 1710682994467,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1689241,
            "range": "± 23405",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 173398,
            "range": "± 985",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58274,
            "range": "± 1018",
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
          "id": "2b9024403f4e989d564751464bb3ee63e1a739c3",
          "message": "Experimental support for holey arrays",
          "timestamp": "2024-02-20T02:40:11Z",
          "url": "https://github.com/y21/dash/pull/81/commits/2b9024403f4e989d564751464bb3ee63e1a739c3"
        },
        "date": 1710683201535,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1671612,
            "range": "± 31211",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 176217,
            "range": "± 1188",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58880,
            "range": "± 833",
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
          "id": "61a0ba34f59e7d1a0b841b1599f8c89ddba113bf",
          "message": "Merge pull request #81 from y21/holey-arrays\n\nExperimental support for holey arrays",
          "timestamp": "2024-03-17T14:58:00+01:00",
          "tree_id": "4893c9320c51a5484c0c5b4ff1fcea2b91dfd1b5",
          "url": "https://github.com/y21/dash/commit/61a0ba34f59e7d1a0b841b1599f8c89ddba113bf"
        },
        "date": 1710684019972,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1672290,
            "range": "± 15916",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 195998,
            "range": "± 1278",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58381,
            "range": "± 347",
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
          "id": "74d4b38b496b175d74f3fe89e9f5fe903c851a2a",
          "message": "fix CI",
          "timestamp": "2024-03-17T16:02:43+01:00",
          "tree_id": "2a3fae15e4baab8e211b40a5855643d319900451",
          "url": "https://github.com/y21/dash/commit/74d4b38b496b175d74f3fe89e9f5fe903c851a2a"
        },
        "date": 1710687988209,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1650486,
            "range": "± 27264",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174485,
            "range": "± 3673",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59109,
            "range": "± 1515",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1691728,
            "range": "± 17373",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37656461,
            "range": "± 1126391",
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
          "id": "9b2f9ba2ced6a3245a54357dedca68b6ce2220a7",
          "message": "add support for global identifier operands to `typeof`",
          "timestamp": "2024-03-17T18:01:49+01:00",
          "tree_id": "49a3f36a3fdd61a86a5b9996347aea4e557dc41f",
          "url": "https://github.com/y21/dash/commit/9b2f9ba2ced6a3245a54357dedca68b6ce2220a7"
        },
        "date": 1710695103309,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1671605,
            "range": "± 24914",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 191103,
            "range": "± 1613",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59511,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1705741,
            "range": "± 14207",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37036456,
            "range": "± 740159",
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
          "id": "5b84a1226b6f5493273351b8c44c39355bffd254",
          "message": "don't use `undefined` if `String` ctor is invoked with no arguments",
          "timestamp": "2024-03-18T18:56:58+01:00",
          "tree_id": "d1742b91dd31b8609f196953dbe767e6942c7a49",
          "url": "https://github.com/y21/dash/commit/5b84a1226b6f5493273351b8c44c39355bffd254"
        },
        "date": 1710784816797,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1651285,
            "range": "± 17395",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 177988,
            "range": "± 2021",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59128,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1713233,
            "range": "± 23240",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37441566,
            "range": "± 721950",
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
          "id": "865659adfa5f23abb972fa2cbdc8c29e8d482ac2",
          "message": "fix tests",
          "timestamp": "2024-03-19T01:54:57+01:00",
          "tree_id": "d45fcf30e7e5a3b37500c0e607a95df6fdb6bb2a",
          "url": "https://github.com/y21/dash/commit/865659adfa5f23abb972fa2cbdc8c29e8d482ac2"
        },
        "date": 1710809891212,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1685827,
            "range": "± 17105",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 176003,
            "range": "± 849",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57946,
            "range": "± 2046",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1708402,
            "range": "± 17615",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36711899,
            "range": "± 698335",
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
          "id": "38a0503e3902d40879f69735d6780e19021f2f6a",
          "message": "Merge pull request #82 from Jacherr/function_apply\n\nadd method `Function#apply`",
          "timestamp": "2024-03-20T23:43:05+01:00",
          "tree_id": "6ea70bb20817ea10e2820deae2cb35e8e0f204e5",
          "url": "https://github.com/y21/dash/commit/38a0503e3902d40879f69735d6780e19021f2f6a"
        },
        "date": 1710974779757,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1676408,
            "range": "± 19398",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 177992,
            "range": "± 862",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58211,
            "range": "± 2854",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1711729,
            "range": "± 16458",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37122296,
            "range": "± 691235",
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
          "id": "99072d0cbc427ba51d1e132e4c82eb8af2106fd3",
          "message": "Merge pull request #83 from Jacherr/minor-regressions\n\nFix small issues in stdlib",
          "timestamp": "2024-03-21T16:27:14+01:00",
          "tree_id": "a370f222957d2f524b6a8d7c01dc5713205d6454",
          "url": "https://github.com/y21/dash/commit/99072d0cbc427ba51d1e132e4c82eb8af2106fd3"
        },
        "date": 1711035030513,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1675055,
            "range": "± 67943",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 193547,
            "range": "± 1070",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58035,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1741563,
            "range": "± 26915",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 38004160,
            "range": "± 523255",
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
          "id": "3732c7298b9a4e07a36d2439a7e0b5cf459a6309",
          "message": "fix rustc and clippy warnings",
          "timestamp": "2024-03-21T23:45:34+01:00",
          "tree_id": "e9f2f7c49360b413ba75352787fcacbe6e0a1a3f",
          "url": "https://github.com/y21/dash/commit/3732c7298b9a4e07a36d2439a7e0b5cf459a6309"
        },
        "date": 1711061330255,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1684601,
            "range": "± 74217",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174303,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57130,
            "range": "± 805",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1729632,
            "range": "± 12933",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 38992842,
            "range": "± 626119",
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
          "id": "382918e80172da0b7708ee6c38c2e6160a7a07ea",
          "message": "ci: only try uploading benchmark output on pushes to master",
          "timestamp": "2024-03-23T02:37:16+01:00",
          "tree_id": "e7a9c32d05cba7f8b46b8cc04d86aec31e598a44",
          "url": "https://github.com/y21/dash/commit/382918e80172da0b7708ee6c38c2e6160a7a07ea"
        },
        "date": 1711158029152,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1660786,
            "range": "± 15934",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174833,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57602,
            "range": "± 2361",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1737023,
            "range": "± 35210",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 38095110,
            "range": "± 855926",
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
          "id": "1ad268c184aa82b4e9b4dfc16b920dff6159029c",
          "message": "Merge pull request #84 from Jacherr/node-compat-run-file\n\nsupport node module file execution",
          "timestamp": "2024-03-23T19:18:45+01:00",
          "tree_id": "e9cb2aac4539266daf1275c970d0761093eadd4d",
          "url": "https://github.com/y21/dash/commit/1ad268c184aa82b4e9b4dfc16b920dff6159029c"
        },
        "date": 1711218119920,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1705703,
            "range": "± 22168",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174616,
            "range": "± 3954",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 56694,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1727814,
            "range": "± 11798",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 39045174,
            "range": "± 756512",
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
          "id": "b46770344dcae0fa562b7195c66b8b9d469a78ef",
          "message": "add a helper for iterating prototypes, deny `Vm::register`",
          "timestamp": "2024-03-24T16:41:53+01:00",
          "tree_id": "533abf8b1b04d8f9be1c4a399a0bf3dafb6d99b3",
          "url": "https://github.com/y21/dash/commit/b46770344dcae0fa562b7195c66b8b9d469a78ef"
        },
        "date": 1711295115103,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1668491,
            "range": "± 26115",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168800,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57460,
            "range": "± 1284",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1738843,
            "range": "± 18346",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37822261,
            "range": "± 1734153",
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
          "id": "f3b0c65b23344ddab6d9b7ae538ed93ca89299a8",
          "message": "trace external vm state",
          "timestamp": "2024-03-29T02:26:02+01:00",
          "tree_id": "8aa84661c4ea41ea902c2bb6f4bfbf293db47b8a",
          "url": "https://github.com/y21/dash/commit/f3b0c65b23344ddab6d9b7ae538ed93ca89299a8"
        },
        "date": 1711675762510,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1665660,
            "range": "± 23414",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171084,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57649,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1717644,
            "range": "± 17066",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37975664,
            "range": "± 661402",
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
          "id": "a9de59f9f91dbdbb23505ed8b85844cf9754e297",
          "message": "make some node modules optional",
          "timestamp": "2024-03-29T02:32:04+01:00",
          "tree_id": "aeed40b82471052321e12293b74f56b8061d6bed",
          "url": "https://github.com/y21/dash/commit/a9de59f9f91dbdbb23505ed8b85844cf9754e297"
        },
        "date": 1711676125796,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1665658,
            "range": "± 37363",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 170316,
            "range": "± 2465",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57670,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1726760,
            "range": "± 43027",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36259451,
            "range": "± 542603",
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
          "id": "582de2c967a7851296c9b7fb9ef6cae8ee31b41c",
          "message": "root async tasks when processing them and add regression tests",
          "timestamp": "2024-03-29T04:17:36+01:00",
          "tree_id": "f1b1433be8eb8ad40202595dba6fa61969b7b37d",
          "url": "https://github.com/y21/dash/commit/582de2c967a7851296c9b7fb9ef6cae8ee31b41c"
        },
        "date": 1711682457040,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1690684,
            "range": "± 8142",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 178740,
            "range": "± 1852",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 57144,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1718955,
            "range": "± 25901",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 38472427,
            "range": "± 632973",
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
          "id": "2bda6e74a59a4bf4c3aade4befe8b4fd72df9ace",
          "message": "implement bare minimum of `require('path')`",
          "timestamp": "2024-03-29T22:15:12+01:00",
          "tree_id": "dbee4edfe6d4e5a0a414e4a98cf2ac20a55112b1",
          "url": "https://github.com/y21/dash/commit/2bda6e74a59a4bf4c3aade4befe8b4fd72df9ace"
        },
        "date": 1711747110837,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1660208,
            "range": "± 9523",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 170498,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55681,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1711634,
            "range": "± 8617",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36494456,
            "range": "± 683523",
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
          "id": "b14403d2cf64afb15f52ed39cf9328c3917d861c",
          "message": "implement basic `EventEmitter` API",
          "timestamp": "2024-03-29T23:03:27+01:00",
          "tree_id": "24a97aa29f36d1ea52094a14382f12cde481c8ba",
          "url": "https://github.com/y21/dash/commit/b14403d2cf64afb15f52ed39cf9328c3917d861c"
        },
        "date": 1711750006019,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1649066,
            "range": "± 21521",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171211,
            "range": "± 4181",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55168,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1725185,
            "range": "± 79469",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37002947,
            "range": "± 799685",
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
          "id": "8ba2f19d391b6423b9de229a46c1ca9ea780a7d8",
          "message": "support computed class members",
          "timestamp": "2024-03-31T04:29:01+02:00",
          "tree_id": "fa0091977957c758a9b1465c05ea8819b1de934b",
          "url": "https://github.com/y21/dash/commit/8ba2f19d391b6423b9de229a46c1ca9ea780a7d8"
        },
        "date": 1711852342189,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1655877,
            "range": "± 40258",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 169709,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 56412,
            "range": "± 1690",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1726713,
            "range": "± 13042",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37222963,
            "range": "± 521705",
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
          "id": "fe0240144179b708afd0aa54b02f6ff41dae821e",
          "message": "support async and generator methods in classes",
          "timestamp": "2024-03-31T04:57:42+02:00",
          "tree_id": "95e6d5e2edd10a9d4ec4554b7289b0e6592b05a1",
          "url": "https://github.com/y21/dash/commit/fe0240144179b708afd0aa54b02f6ff41dae821e"
        },
        "date": 1711854065603,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1691056,
            "range": "± 193609",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 167329,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 56125,
            "range": "± 1254",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1741394,
            "range": "± 18643",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37905657,
            "range": "± 575510",
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
          "id": "176c91c140fe4c5bffaf8ffd52067c63c92506a2",
          "message": "implement `in` keyword",
          "timestamp": "2024-03-31T14:54:54+02:00",
          "tree_id": "b245e68d7047cd135a59bada3c3a3bf479ef88e2",
          "url": "https://github.com/y21/dash/commit/176c91c140fe4c5bffaf8ffd52067c63c92506a2"
        },
        "date": 1711889890335,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1669470,
            "range": "± 17391",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 175690,
            "range": "± 1551",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55942,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1732025,
            "range": "± 17627",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37275570,
            "range": "± 624305",
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
          "id": "fd9dacd67eda828ccf242921ac3c7ccb144f258e",
          "message": "clear symbol list when dropping `LocalScope`",
          "timestamp": "2024-04-13T21:45:18+02:00",
          "tree_id": "2a35d41e927c8f41301d0252d72b3e08c103b04f",
          "url": "https://github.com/y21/dash/commit/fd9dacd67eda828ccf242921ac3c7ccb144f258e"
        },
        "date": 1713037714366,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1651330,
            "range": "± 16229",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 175069,
            "range": "± 1412",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55431,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1730462,
            "range": "± 23842",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36973565,
            "range": "± 454565",
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
          "id": "9d9f072bc5e82a982ddc86ee89b44d99f79ef098",
          "message": "update test262 pass rate",
          "timestamp": "2024-05-01T00:20:03+02:00",
          "tree_id": "b4734bdaddc4858465f3d8af599c2edab121367e",
          "url": "https://github.com/y21/dash/commit/9d9f072bc5e82a982ddc86ee89b44d99f79ef098"
        },
        "date": 1714515808024,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1671444,
            "range": "± 28893",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 170670,
            "range": "± 5108",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55751,
            "range": "± 3641",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1736559,
            "range": "± 24027",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37198832,
            "range": "± 507341",
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
          "id": "16273128375362b24b0c3d1b64d5f17773635f70",
          "message": "properly evaluate spread operator in array literals",
          "timestamp": "2024-05-04T17:53:04+02:00",
          "tree_id": "30fd2fc1dd142ef8654ccb7adc8211e14a6833c3",
          "url": "https://github.com/y21/dash/commit/16273128375362b24b0c3d1b64d5f17773635f70"
        },
        "date": 1714838177103,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1576000,
            "range": "± 20710",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 173896,
            "range": "± 910",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 55537,
            "range": "± 2572",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1513611,
            "range": "± 17058",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36526827,
            "range": "± 763936",
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
          "id": "25d269bbf8f7693e15627f649574606c81142505",
          "message": "support dynamic getters in classes",
          "timestamp": "2024-05-05T20:03:51+02:00",
          "tree_id": "8fc642eafaffa39f33fe19f932a374b52787bf23",
          "url": "https://github.com/y21/dash/commit/25d269bbf8f7693e15627f649574606c81142505"
        },
        "date": 1714932456679,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1828532,
            "range": "± 61094",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174087,
            "range": "± 2667",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59973,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2060132,
            "range": "± 23766",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37013385,
            "range": "± 606631",
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
          "id": "5d7c32b9fa0b3ab8d770b6d5f99a9dd7cc81bdb8",
          "message": "don't treat `get() {}` a getter in classes",
          "timestamp": "2024-05-05T21:52:46+02:00",
          "tree_id": "46618dfa0936b69a1cd4ef8abc1974e52c84c585",
          "url": "https://github.com/y21/dash/commit/5d7c32b9fa0b3ab8d770b6d5f99a9dd7cc81bdb8"
        },
        "date": 1714938953897,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1690468,
            "range": "± 17141",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 160630,
            "range": "± 3027",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 54122,
            "range": "± 2008",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 1906318,
            "range": "± 23496",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 34291119,
            "range": "± 1592285",
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
          "id": "5b4613a4f82df55a3e3dc049307646aa3d2bc00e",
          "message": "support class expressions",
          "timestamp": "2024-05-05T22:48:17+02:00",
          "tree_id": "e4d6426e465c9ba31e3802ac5441ebccc08f62e1",
          "url": "https://github.com/y21/dash/commit/5b4613a4f82df55a3e3dc049307646aa3d2bc00e"
        },
        "date": 1714942293392,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1786167,
            "range": "± 61012",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 170071,
            "range": "± 1340",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59681,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2062923,
            "range": "± 9890",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 37088991,
            "range": "± 616399",
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
          "id": "44a02550d314e6ed86cb196aa1c38c87608d6760",
          "message": "implement `Object.setPrototypeOf` and add `global`",
          "timestamp": "2024-05-06T00:48:58+02:00",
          "tree_id": "d44b3249ee4edd3519e3bfe3cbf230a6b23f63ba",
          "url": "https://github.com/y21/dash/commit/44a02550d314e6ed86cb196aa1c38c87608d6760"
        },
        "date": 1714949628937,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1801918,
            "range": "± 16345",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168017,
            "range": "± 2448",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 60539,
            "range": "± 2681",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2057804,
            "range": "± 10084",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36861712,
            "range": "± 395301",
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
          "id": "c175698d5787a2cbd35f7447ec003199f5808290",
          "message": "remove `StaticPropAccessW` and make the existing one wide",
          "timestamp": "2024-05-06T01:03:16+02:00",
          "tree_id": "69d482518c2bc67cb51c70969eb57daa36e3816e",
          "url": "https://github.com/y21/dash/commit/c175698d5787a2cbd35f7447ec003199f5808290"
        },
        "date": 1714950429968,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1796593,
            "range": "± 19688",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 172375,
            "range": "± 1016",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59760,
            "range": "± 2463",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2056981,
            "range": "± 21783",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 35733638,
            "range": "± 655057",
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
          "id": "2bb12d53e18b44341da9a2dc14a996d30010dc70",
          "message": "Merge pull request #88 from y21/try-finally\n\nImplement try-finally blocks",
          "timestamp": "2024-05-11T21:43:11+02:00",
          "tree_id": "8caf79411e1dcad92afb201c12f25d69d79c03cc",
          "url": "https://github.com/y21/dash/commit/2bb12d53e18b44341da9a2dc14a996d30010dc70"
        },
        "date": 1715456779556,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1804564,
            "range": "± 19624",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 174936,
            "range": "± 5539",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 60306,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2057328,
            "range": "± 4791",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 35774967,
            "range": "± 499432",
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
          "id": "adddfb43ddf0d574a19fc28a60ff8eef1a67d7da",
          "message": "support try/finally in generators & async fns",
          "timestamp": "2024-05-12T02:09:53+02:00",
          "tree_id": "4a00d6ae22f75405ddab1943d203a8018e17b86b",
          "url": "https://github.com/y21/dash/commit/adddfb43ddf0d574a19fc28a60ff8eef1a67d7da"
        },
        "date": 1715472788058,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1762921,
            "range": "± 22453",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171667,
            "range": "± 1925",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59519,
            "range": "± 1829",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2064447,
            "range": "± 10082",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 33746426,
            "range": "± 416359",
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
          "id": "d46332755895c5e5bd84dd87e70d7ec73460f013",
          "message": "support default parameters in closures\n\nfixes #89",
          "timestamp": "2024-05-12T04:32:42+02:00",
          "tree_id": "766eb1eff5d888965e86a28e837f2fe2eae740ac",
          "url": "https://github.com/y21/dash/commit/d46332755895c5e5bd84dd87e70d7ec73460f013"
        },
        "date": 1715481350458,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1779051,
            "range": "± 28765",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 168626,
            "range": "± 896",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 58361,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2047637,
            "range": "± 5822",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36172504,
            "range": "± 294826",
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
          "id": "c74fe21a802616a823531c76ce4f624e968727ae",
          "message": "Merge pull request #91 from hamirmahal/fix/usage-of-node12-in-actions/checkout\n\nfix: usage of `node12` in `actions/checkout`",
          "timestamp": "2024-06-10T10:02:03+02:00",
          "tree_id": "90f386b2648dc582d0fbad78460710eb99d8cf01",
          "url": "https://github.com/y21/dash/commit/c74fe21a802616a823531c76ce4f624e968727ae"
        },
        "date": 1718006705243,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1785618,
            "range": "± 19668",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 171237,
            "range": "± 1000",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 59172,
            "range": "± 1714",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2055140,
            "range": "± 10559",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36635208,
            "range": "± 419075",
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
          "id": "ba9684c3b9a4403a311ee93886de06e87c29e750",
          "message": "exclude broken crates for now",
          "timestamp": "2024-06-10T21:46:36+02:00",
          "tree_id": "435a696afd847c55794ca3a817246d0e77360efe",
          "url": "https://github.com/y21/dash/commit/ba9684c3b9a4403a311ee93886de06e87c29e750"
        },
        "date": 1718049188047,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1750409,
            "range": "± 18656",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 166585,
            "range": "± 3658",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 60806,
            "range": "± 5041",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2072666,
            "range": "± 10688",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 32937160,
            "range": "± 515724",
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
          "id": "b9c207b53aa675c1ac2f4dc4f3ae7379f3bae612",
          "message": "replace `expect_and_skip` with more general `eat`",
          "timestamp": "2024-07-14T12:42:51+02:00",
          "tree_id": "f0e890cf4ec3d4f073c5ff362e3d5028d631c4f0",
          "url": "https://github.com/y21/dash/commit/b9c207b53aa675c1ac2f4dc4f3ae7379f3bae612"
        },
        "date": 1720953968170,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1789962,
            "range": "± 19460",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 169004,
            "range": "± 1454",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 60985,
            "range": "± 1529",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2052152,
            "range": "± 9006",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 36467937,
            "range": "± 466257",
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
          "id": "d9db3e435e7aeb4261c2b870abb2eddbf6fd6ff1",
          "message": "remove unnecessary indirection from `DispatchContext`",
          "timestamp": "2024-07-15T23:24:14+02:00",
          "tree_id": "4d7c66358edf322514090f3df9a1a28087f1f2bd",
          "url": "https://github.com/y21/dash/commit/d9db3e435e7aeb4261c2b870abb2eddbf6fd6ff1"
        },
        "date": 1721078847885,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1769349,
            "range": "± 77678",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 166868,
            "range": "± 1509",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 60702,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2050798,
            "range": "± 11971",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 35879946,
            "range": "± 514384",
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
          "id": "825ffe4d75d64c770fbd7b68e495c57ca15d852b",
          "message": "allow skipping elements in array destructuring pattern",
          "timestamp": "2024-07-16T23:54:27+02:00",
          "tree_id": "d3d36349f9eb5c129c460d60b838ab2416b38580",
          "url": "https://github.com/y21/dash/commit/825ffe4d75d64c770fbd7b68e495c57ca15d852b"
        },
        "date": 1721167118679,
        "tool": "cargo",
        "benches": [
          {
            "name": "interpreter",
            "value": 1770328,
            "range": "± 47319",
            "unit": "ns/iter"
          },
          {
            "name": "fib_recursive(12)",
            "value": 167492,
            "range": "± 9697",
            "unit": "ns/iter"
          },
          {
            "name": "fib_iterative(12)",
            "value": 60916,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "parse+compile tinycolor2",
            "value": 2051207,
            "range": "± 8188",
            "unit": "ns/iter"
          },
          {
            "name": "exec tinycolor2 parse hex+toFilter",
            "value": 35503195,
            "range": "± 406067",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}