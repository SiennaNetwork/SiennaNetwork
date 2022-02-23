import { Address } from '../core'
import { Permit } from '../permit'

export type AuthMethod<T> =
    | {
        permit: Permit<T>;
    }
    | {
        viewing_key: {
            address: Address;
            key: string;
        };
    };
