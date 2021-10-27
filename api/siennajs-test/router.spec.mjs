import debug from "debug";
import { assert } from "chai";

import siennajs from "../siennajs/index";

const Assembler = siennajs.hop.Assembler;
const get_path = siennajs.hop.get_path;

function create_token(id) {
  if (id) {
    return {
      id: id,
    //   token: {
    //     Snip20Data: {
    //       address: id,
    //       code_hash: "",
    //     },
    //   },
    };
  }

  return {
    id: "Scrt",
    // token: { Scrt: {} },
  };
}

function create_pair(a, b) {
    return {
        A: a,
        B: b,
        // pair_address: `${parseInt(Math.random() * 1000)}-address`,
        // pair_code_hash: "",
    };
}

describe("Test assembler in creating a route", function () {
  it("Can instantiate assembler with given tokens and pairs", async function () {
      const A = create_token();
      const B = create_token("token-3");

      const pairs = [
          create_pair(create_token(), create_token("token-2")),
          create_pair(create_token("token-2"), create_token("token-3")),
          create_pair(create_token("token-3"), create_token("token-4")),
          create_pair(create_token("token-2"), create_token("token-8")),
          create_pair(create_token("token-8"), create_token("token-18")),
          create_pair(create_token("token-18"), create_token("token-3")),
          create_pair(create_token(), create_token("token-4")),
      ];

      const assembler = new Assembler(pairs);

      console.log(JSON.stringify(assembler.get_hops(A, B), null, 2));
      
    //   console.log(get_path(pairs, A, B, 20000));
  });
});
