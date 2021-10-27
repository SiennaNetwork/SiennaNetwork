import { Address, TokenType, TokenPair } from "./core";

export interface Snip20Data {
    Snip20Data: {
        address: Address;
        code_hash: String;
    };
}

export interface Scrt {
    Scrt: {};
}

export enum Token {
    Snip20Data,
    Scrt,
}

export interface TokenComparable {
    id: string | Address; // Here should be token address, or 'Scrt'
    token: Token;
}

export interface TokenPairExtended {
    A: TokenComparable;
    B: TokenComparable;
    pair_address: Address;
    pair_code_hash: string;
    hops?: TokenPairExtended[];
    destinations?: string[];
}

export interface TokenPairTree {
    A: TokenComparable;
    B: TokenComparable;
    pair_address: Address;
    pair_code_hash: string;
    next?: TokenPairTree,
    solved: boolean
}

export interface Hop {
    from_token: Token;
    pair_address: Address;
    pair_code_hash: string;
}

function map_destinations(pairs: TokenPairExtended[]): TokenPairExtended[] {
    for (let i = 0; i < pairs.length; i++) {
        pairs[i].destinations = pairs[i].destinations || [pairs[i].B.id];

        if (pairs[i].hops?.length) {
            pairs[i].destinations = pairs[i].destinations?.concat(get_destinations(pairs[i].hops || []));
        }
    }

    return pairs;
}

function get_destinations(pairs: TokenPairExtended[]): string[] {
    let destinations: string[] = [];

    for (let i = 0; i < pairs.length; i++) {
        pairs[i].destinations = pairs[i].destinations || [pairs[i].B.id];

        if (pairs[i].hops?.length) {
            pairs[i].destinations = pairs[i].destinations?.concat(get_destinations(pairs[i].hops || []));
        }

        destinations = destinations.concat(pairs[i].destinations || []);
    }

    return destinations;
}

export class Assembler {
    private pairs: TokenPairExtended[] = [];

    constructor(pairs: TokenPairExtended[]) {
        const secondary: TokenPairExtended[] = [];

        for (const pair of pairs) {
            secondary.push({
                ...pair,
                B: pair.A,
                A: pair.B,
            });
        }

        pairs = pairs.concat(secondary);

        this.pairs = pairs;
    }

    get_hops(A: TokenComparable, B: TokenComparable): TokenPairExtended[] {
        if (A.id === B.id) {
            throw new Error("Token A and token B must be different");
        }
        
        const paths = this.pairs.filter(p => p.A.id === A.id);

        for (const path of paths) {
            const filtered = this.pairs.filter(p => p.A.id !== path.A.id && p.B.id !== path.B.id);

            path.hops = this.nest(path, filtered);
        }

        return map_destinations(paths)
            .filter(p => p.destinations?.includes(B.id))
            .sort((a, b) => (a.destinations || []).length - (b.destinations || []).length);
    }

    private nest(parent: TokenPairExtended, pairs: TokenPairExtended[]): TokenPairExtended[] {
        const hops = pairs.filter(child => parent.B.id === child.A.id);

        return hops
            .map(hop => ({
                ...hop,
                hops: this.nest(hop, pairs.filter(p => p.A.id !== hop.A.id && p.B.id !== hop.B.id)),
            }));
    }
}
