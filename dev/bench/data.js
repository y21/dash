window.BENCHMARK_DATA = {
  "lastUpdate": 1676771791006,
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
      }
    ]
  }
}