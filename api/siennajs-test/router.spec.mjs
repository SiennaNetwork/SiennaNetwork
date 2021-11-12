import debug from "debug";
import { assert } from "chai";

import siennajs from "../siennajs/index";

const Assembler = siennajs.hop.Assembler;
const TokenPair = siennajs.hop.TokenPair;

function create_token(id) {
  if (id) {
    return {
        custom_token: {
          contract_addr: id,
          token_code_hash: "",
        },
      };
  }

  return { native_token: { denom: "uscrt" } };
}

function create_pair(a, b) {
  return new TokenPair(a, b, `${parseInt(Math.random() * 1000)}-address`, "");
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
      const hops = tree.into_hops();
      const printout = tree.printout();

      console.log(JSON.stringify(tree, null, 2));
      console.log(JSON.stringify(hops, null, 2));
      console.log(printout.join(" ==> "));

      assert.strictEqual(printout.join(" ==> "), "token-2 -> token-3 ==> token-3 -> token-4");
  });
});