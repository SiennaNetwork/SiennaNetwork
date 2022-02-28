import { Address, ViewingKey } from '../core'
import { Permit, Signer } from '../permit'

export type AuthMethod<T> =
    | {
        permit: Permit<T>;
    }
    | {
        viewing_key: {
            address: Address;
            key: ViewingKey;
        };
    };

export class LendAuth {
    private constructor(private readonly strategy: AuthStrategy) { }

    static viewing_key(address: Address, key: ViewingKey): LendAuth {
        return new this({
            type: 'vk',
            viewing_key: {
                address,
                key
            }
        })
    }

    static permit(signer: Signer): LendAuth {
        return new this({
            type: 'permit',
            signer
        })
    }

    async create_method<T>(address: Address, permission: T): Promise<AuthMethod<T>> {
        if (this.strategy.type === 'permit') {
            const permit = await this.strategy.signer.sign({
                permit_name: `SiennaJS permit for ${address}`,
                allowed_tokens: [ address ],
                permissions: [ permission ]
            })

            return {
                permit
            }
        } else {
            return {
                viewing_key: this.strategy.viewing_key
            }
        }
    }
}

type AuthStrategy =
    | {
        type: 'permit',
        signer: Signer
    }
    | {
        type: 'vk',
        viewing_key: {
            address: Address;
            key: ViewingKey;
        };
    };
