import { Address } from "./core";

/**
 * Interface for any Snip20 token
 */
export interface Snip20Data {
    Snip20Data: {
        address: Address;
        code_hash: String;
    };
}

/**
 * Interface for native token
 */
export interface Scrt {
    Scrt: {};
}

/**
 * Enum of token representation used in the swap router contract
 */
export enum Token {
    Snip20Data,
    Scrt,
}

/**
 * This is internal interface wrapper around the Token
 * it helps us to compare them easily by using the id
 */
export interface TokenComparable {
    id: string | Address; // Here should be token address, or 'Scrt'
    token: Token;
}

/**
 * Simple token pair interface for easier filling in the pairs
 */
export interface TokenPairSimple {
    A: Token;
    B: Token;
    pair_address: Address;
    pair_code_hash: string;
}

/**
 * Token pair comparable that lets us compare tokens and run filter
 * easily within Assembler class and its functions
 */
export interface TokenPairComparable {
    A: TokenComparable;
    B: TokenComparable;
    pair_address: Address;
    pair_code_hash: string;
}

/**
 * Token pair comparable tree that spans until its solved or broken.
 * IMPORTANT: Only the last node will be marked as solved (performance fix),
 * so to check if its solved you'll have to use the function provided in this file.
 */
export interface TokenPairComparableTree {
    A: TokenComparable;
    B: TokenComparable;
    pair_address: Address;
    pair_code_hash: string;
    next?: TokenPairComparableTree,
    solved: boolean,
    printout?: string[],
}

/**
 * Hop representation used in the router contract,
 * array of theese objects needs to be passed to the router
 * in order for it to execute the transaction which will
 * swap any for any token.
 * 
 * IMPORTANT: Router gives only one constraint: Scrt can be first or last in the chain
 * it cannot appear anywhere in the middle of the chain.
 */
export interface Hop {
    from_token: Token;
    pair_address: Address;
    pair_code_hash: string;
}

/**
 * Helper for creating a comparable token pair from all its information
 * 
 * @param {Token} A 
 * @param {Token} B 
 * @param {Address} pair_address 
 * @param {string} pair_code_hash 
 * @returns {TokenPairComparable}
 */
export function into_token_pair_extended(A: Token, B: Token, pair_address: Address, pair_code_hash: string): TokenPairComparable {
    return {
        A: token_into_comparable(A),
        B: token_into_comparable(B),
        pair_address,
        pair_code_hash,
    };
}

/**
 * Converts token enum into a comparable token
 * 
 * @param {Token} token 
 * @returns {TokenComparable}
 */
export function token_into_comparable(token: Token): TokenComparable {
    let id = "Scrt";

    if (
        token.hasOwnProperty("Snip20Data") &&
        typeof (token as unknown as Snip20Data).Snip20Data === "object" &&
        (token as unknown as Snip20Data).Snip20Data
    ) {
        id = (token as unknown as Snip20Data).Snip20Data.address;
    }

    return {
        id,
        token,
    };
}

/**
 * Converts simple token pair into a comparable token pair
 * 
 * @param {TokenPairSimple | TokenPairComparable} token_pair 
 * @returns {TokenPairComparable}
 */
export function token_pair_simple_into_token_pair_comparable(token_pair: TokenPairSimple | TokenPairComparable): TokenPairComparable {
    // @ts-ignore
    if (typeof token_pair.A.id !== "undefined" && typeof token_pair.B.id !== "undefined") {
        return token_pair as TokenPairComparable;
    }

    return {
        ...token_pair,
        A: token_into_comparable(token_pair.A as Token),
        B: token_into_comparable(token_pair.B as Token),
    }
}

/**
 * Creates comparable token pair
 * 
 * @param {TokenPairComparable} pair 
 * @returns {TokenPairComparableTree}
 */
export function token_pair_comparable_into_token_pair_comparable_tree(pair: TokenPairComparable): TokenPairComparableTree {
    return {
        ...pair,
        solved: false,
    };
}

/**
 * Converts the single comparable token pair into a hop
 * 
 * @param {TokenPairComparable} pair 
 * @returns {Hop}
 */
export function token_pair_comparable_into_hop(pair: TokenPairComparable): Hop {
    return {
        from_token: pair.A.token,
        pair_address: pair.pair_address,
        pair_code_hash: pair.pair_code_hash,
    }
}

/**
 * Converts solved tree into an array of hops that can be then used for the swap router
 * 
 * @param {TokenPairComparableTree} pair 
 * @returns {Hop[]}
 */
export function token_pair_tree_into_hops(pair: TokenPairComparableTree): Hop[] {
    if (!pair.solved) {
        throw new Error("Token path is not solved, cannot create hops");
    }

    const hops = [];
    let run = true;
    let node = pair;

    hops.push(token_pair_comparable_into_hop(node));

    while (run === true) {
        if (typeof node.next !== "undefined") {
            hops.push(token_pair_comparable_into_hop(node.next));
            node = node.next;
        }
        else {
            run = false;
        }
    }

    return hops;
}

/**
 * Check if the tree is considered solved (if the last node is solved)
 * 
 * @param {TokenPairComparableTree} tree 
 * @returns {boolean}
 */
export function token_pair_comparable_tree_chain_solved(tree: TokenPairComparableTree): boolean {
    let node: TokenPairComparableTree | undefined = tree;
    let first = true;

    while (node) {
        // IMPORTANT: This will add the constraint that is presented
        // in the router contract: Scrt can only be first or last part
        // of the hop chain. It cannot appear anywhere in the middle
        // of the chain.
        if (!first && node.A.id === "Scrt") {
            return false;
        }

        first = false;

        if (node?.solved) {
            return true;
        }

        node = node.next;
    }

    return false;
}

/**
 * Returns an array that has each step printed in human-readeable format
 * 
 * @param {TokenPairComparableTree} tree 
 * @returns {string[]}
 */
export function print_token_pair_comparable_tree(tree: TokenPairComparableTree): string[] {
    let node: TokenPairComparableTree | undefined = tree;
    let printout: string[] = [];

    while (node) {
        printout.push(`${node.A.id} -> ${node.B.id}`);
        node = node.next;
    }

    return printout;
}

/**
 * This class will essentially do all the work for generating the path of exchange
 * @class {Assembler}
 */
export class Assembler {
    private A?: TokenComparable;
    private B?: TokenComparable;
    private pairs: TokenPairComparableTree[] = [];
    private parents: TokenPairComparableTree[] = [];

    constructor(pairs: TokenPairSimple[] | TokenPairComparable[], private filter: string[] = []) {
        for (const pair of pairs) {
            let pair_comparable = token_pair_comparable_into_token_pair_comparable_tree(
                token_pair_simple_into_token_pair_comparable(pair)
            );

            this.pairs.push(pair_comparable);
            this.pairs.push({
                ...pair_comparable,
                A: pair_comparable.B,
                B: pair_comparable.A,
            });
        }
    }

    /**
     * Set token that we want to swap
     * 
     * @param {Token} token 
     * @returns {Assembler}
     */
    from(token: Token): this {
        this.A = token_into_comparable(token);

        return this;
    }

    /**
     * Set token that we want to get after the swap
     * 
     * @param {Token} token 
     * @returns {Assembler}
     */
    to(token: Token): this {
        if (!this.A) {
            throw new Error("You'll have to provide the 'from' token first");
        }

        this.B = token_into_comparable(token);

        return this.prepare();
    }

    /**
     * Prepare this class so that everything is validated and all the 
     * parents and left over pairs are set.
     * 
     * @returns {Assembler}
     */
    private prepare(): this {
        if ((!this.A || !this.B) || (this.A?.id === this.B?.id)) {
            throw new Error("Invalid pair provided");
        }

        // Parents can be any of the pairs that have A as A
        this.parents = this.pairs.filter(p => {
            return p.A.id === this.A?.id;
        }).map(p => token_pair_comparable_into_token_pair_comparable_tree(p));

        this.filter.push(this.A?.id);

        // Left over pairs cannot have A in either A or B
        this.pairs = this.pairs.filter(p => {
            return !this.filter.includes(p.A.id) && !this.filter.includes(p.B.id);
        });

        return this;
    }

    /**
     * Create token tree that will let us know what paths we need to take in order
     * to swap A for B
     * 
     * @returns {TokenPairComparableTree}
     */
    get_tree(): TokenPairComparableTree {
        if ((!this.A || !this.B) || (this.A?.id === this.B?.id)) {
            throw new Error("Invalid pair provided");
        }

        // First, we will try to find a perfect match with a single pair
        for (const parent of this.parents) {
            if (parent.A.id === this.A?.id && parent.B.id === this.B?.id) {
                return { ...token_pair_comparable_into_token_pair_comparable_tree(parent), solved: true };
            }
        }

        // Try to finish each parents tree
        for (let i = 0; i < this.parents.length; i++) {
            try {
                const assembler = (new Assembler(this.pairs, this.filter)).from(this.parents[i].B.token).to(this.B.token);

                this.filter = assembler.filter;

                this.parents[i].next = assembler.get_tree();

                this.parents[i].solved = token_pair_comparable_tree_chain_solved(this.parents[i]);
            }
            catch (e) { }
        }

        // Filter parents to only have the solved ones left and then sort is so that the 
        // first in line is the one with the least amount of hops in the chain
        this.parents = this.parents
            .filter(p => p.solved)
            .sort((a, b) => print_token_pair_comparable_tree(a).length - print_token_pair_comparable_tree(b).length);

        if (!this.parents.length) {
            throw new Error("No possible solution for given pair");
        }

        return this.parents[0];
    }

    /**
     * Return the list of all the hops needed to execute the swap
     * 
     * @returns {Hop[]}
     */
    get(): Hop[] {
        return token_pair_tree_into_hops(this.get_tree());
    }
}
