import { Address, TypeOfToken, CustomToken, NativeToken, get_type_of_token_id } from "./core";

/**
 * Simple token pair interface for easier filling in the pairs
 */
export class TokenPair {
    constructor(
        public A: TypeOfToken,
        public B: TypeOfToken,
        public pair_address: Address,
        public pair_code_hash: string
    ) { }

    a_id(): string {
        return get_type_of_token_id(this.A);
    }

    b_id(): string {
        return get_type_of_token_id(this.B);
    }

    eq(other: TokenPair): boolean {
        return this.a_id() === other.a_id() && this.b_id() === other.b_id();
    }

    into_hop(): Hop {
        return {
            from_token: this.A,
            pair_address: this.pair_address,
            pair_code_hash: this.pair_code_hash,
        };
    }

    /**
     * Converts itself to TokenPairTree if not already it.
     * 
     * @returns {TokenPairTree}
     */
    into_tree(): TokenPairTree {
        if (this instanceof TokenPairTree) {
            return this;
        }

        return new TokenPairTree(
            this.A,
            this.B,
            this.pair_address,
            this.pair_code_hash,
        );
    }
}

/**
 * Token pair comparable tree that spans until its solved or broken.
 * IMPORTANT: Only the last node will be marked as solved (performance fix),
 * so to check if its solved you'll have to use the function provided in this file.
 */
export class TokenPairTree extends TokenPair {
    next?: TokenPairTree;
    public is_solved: boolean = false;

    /**
     * Check if the tree is considered solved (if the last node is solved)
     * 
     * @returns {boolean}
     */
    solved(): boolean {
        let node: TokenPairTree | undefined = this;
        let first = true;

        while (node) {
            // IMPORTANT: This will add the constraint that is presented
            // in the router contract: Scrt can only be first or last part
            // of the hop chain. It cannot appear anywhere in the middle
            // of the chain.
            if (!first && node.a_id() === "native") {
                return false;
            }

            first = false;

            if (node?.is_solved) {
                this.is_solved = true;
                return true;
            }

            node = node.next;
        }

        return false;
    }

    /**
     * Converts solved tree into an array of hops that can be then used for the swap router
     * 
     * @returns {Hop[]}
     */
    into_hops(): Hop[] {
        if (!this.solved()) {
            throw new Error("Token path is not solved, cannot create hops");
        }

        const hops = [];
        let node: TokenPairTree | undefined = this;

        hops.push(node.into_hop());

        while (node) {
            if (node.next) {
                hops.push(node.next?.into_hop());
            }

            node = node.next;
        }

        return hops;
    }

    /**
     * Returns an array that has each step printed in human-readeable format
     * 
     * @returns {string[]}
     */
    printout(): string[] {
        let node: TokenPairTree | undefined = this;
        let printout: string[] = [];

        while (node) {
            printout.push(`${node.a_id()} -> ${node.b_id()}`);
            node = node.next;
        }

        return printout;
    }

    /**
     * Creates itself into a reverse exchange pair.
     * 
     * @returns {TokenPairTree}
     */
    into_reverse(): TokenPairTree {
        return new TokenPairTree(
            this.B,
            this.A,
            this.pair_address,
            this.pair_code_hash,
        );
    }
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
    from_token: TypeOfToken;
    pair_address: Address;
    pair_code_hash: string;
}

/**
 * This class will essentially do all the work for generating the path of exchange
 * @class {Assembler}
 */
export class Assembler {
    private A?: TypeOfToken;
    private a_id?: string;
    private B?: TypeOfToken;
    private b_id?: string;
    private pairs: TokenPairTree[] = [];
    private parents: TokenPairTree[] = [];

    constructor(pairs: TokenPair[] | TokenPairTree[], private filter: string[] = []) {
        for (const pair of pairs) {
            let _pair: any = pair;

            if (
                !(_pair instanceof TokenPair) &&
                !(_pair instanceof TokenPairTree)
            ) {
                _pair = new TokenPair(
                    pair.A,
                    pair.B,
                    pair.pair_address,
                    pair.pair_code_hash
                );
            }

            const tree = _pair.into_tree();

            this.pairs.push(tree);
            this.pairs.push(tree.into_reverse());
        }
    }

    /**
     * Set token that we want to swap
     * 
     * @param {TypeOfToken} token 
     * @returns {Assembler}
     */
    from(token: TypeOfToken): this {
        this.A = token;
        this.a_id = get_type_of_token_id(token);

        return this;
    }

    /**
     * Set token that we want to get after the swap
     * 
     * @param {TypeOfToken} token 
     * @returns {Assembler}
     */
    to(token: TypeOfToken): this {
        if (!this.A) {
            throw new Error("Swap path assembler: You'll have to provide the 'from' token first");
        }

        this.B = token;
        this.b_id = get_type_of_token_id(token);

        if (this.a_id === this.b_id) {
            throw new Error("Swap path assembler: Provided tokens are the same token");
        }

        return this.prepare();
    }

    /**
     * Prepare this class so that everything is validated and all the 
     * parents and left over pairs are set.
     * 
     * @returns {Assembler}
     */
    private prepare(): this {
        if ((!this.A || !this.B)) {
            throw new Error("Swap path assembler: You'll have to provide pairs first");
        }

        // Parents can be any of the pairs that have A as A
        this.parents = this.pairs.filter(p => {
            return p.a_id() === this.a_id;
        });

        this.filter.push(this.a_id as string);

        // Left over pairs cannot have A in either A or B
        this.pairs = this.pairs.filter(p => {
            return !this.filter.includes(p.a_id()) && !this.filter.includes(p.b_id());
        });

        return this;
    }

    /**
     * Create token tree that will let us know what paths we need to take in order
     * to swap A for B
     * 
     * @returns {TokenPairTree}
     */
    get_tree(): TokenPairTree {
        if ((!this.A || !this.B)) {
            throw new Error("Swap path assembler: You'll have to provide pairs first");
        }

        // First, we will try to find a perfect match with a single pair
        for (const parent of this.parents) {
            if (parent.a_id() === this.a_id && parent.b_id() === this.b_id) {
                parent.is_solved = true;
                return parent;
            }
        }

        // Try to finish each parents tree
        for (let i = 0; i < this.parents.length; i++) {
            try {
                const assembler = (new Assembler(this.pairs, this.filter)).from(this.parents[i].B).to(this.B);

                this.filter = assembler.filter;

                this.parents[i].next = assembler.get_tree();

                this.parents[i].is_solved = this.parents[i].solved();
            }
            catch (e) { }
        }

        // Filter parents to only have the solved ones left and then sort is so that the 
        // first in line is the one with the least amount of hops in the chain
        this.parents = this.parents
            .filter(p => p.is_solved)
            .sort((a, b) => a.printout().length - b.printout().length);

        if (!this.parents.length) {
            throw new Error("Swap path assembler: No possible solution for given pair");
        }

        return this.parents[0];
    }
}
