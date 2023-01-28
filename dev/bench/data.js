window.BENCHMARK_DATA = {
  "lastUpdate": 1674877374460,
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
      }
    ]
  }
}