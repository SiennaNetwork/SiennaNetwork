import debug from "debug";
import { assert } from "chai";

import siennajs from "../siennajs/index";

const Assembler = siennajs.hop.Assembler;
const print_token_pair_comparable_tree = siennajs.hop.print_token_pair_comparable_tree;

function create_token(id) {
  if (id) {
    return {
        Snip20Data: {
          address: id,
          code_hash: "",
        },
      };
  }

  return { Scrt: {} };
}

function create_pair(a, b) {
    return {
        A: a,
        B: b,
        pair_address: `${parseInt(Math.random() * 1000)}-address`,
        pair_code_hash: "",
    };
}

describe("Test assembler in creating a route", function () {
  it("Can instantiate assembler with given tokens and pairs", async function () {
      const A = create_token("token-2");
      const B = create_token("token-4");

      const pairs = [
          create_pair(create_token(), create_token("token-2")),
          create_pair(create_token("token-2"), create_token("token-3")),
          create_pair(create_token("token-3"), create_token("token-4")),
          create_pair(create_token("token-2"), create_token("token-8")),
          create_pair(create_token("token-18"), create_token("token-5")),
          create_pair(create_token("token-8"), create_token("token-18")),
          create_pair(create_token("token-18"), create_token("token-3")),
          create_pair(create_token(), create_token("token-4")),
      ];

      const assembler = new Assembler(pairs);
      const tree = assembler.from(A).to(B).get_tree();
      const hops = assembler.get();
      const printout = print_token_pair_comparable_tree(tree);

    //   console.log(JSON.stringify(hops, null, 2));
    //   console.log(printout.join(" ==> "));

      assert.strictEqual(printout.join(" ==> "), "token-2 -> token-3 ==> token-3 -> token-4");
  });
});