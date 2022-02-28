import { Executor } from '../contract'
import { create_entropy, decode_data, ViewingKey } from '../core'

import { Tx } from 'secretjs'

const CREATE_GAS = '50000'
const SET_GAS = '40000'

export class ViewingKeyExecutor extends Executor {
    async create_viewing_key(
        padding?: string | null,
    ): Promise<{result: Tx, key: ViewingKey}> {
        const msg = {
            create_viewing_key: {
                entropy: create_entropy(),
                padding,
            }
        }

        const result = await this.run(
            msg,
            CREATE_GAS
        )

        return {
            result,
            key: result.code == 0 ? decode_vk(result) : ''
        }
    }

    async set_viewing_key(
        key: ViewingKey,
        padding?: string | null,
    ): Promise<Tx> {
        const msg = {
            set_viewing_key: {
                key,
                padding
            }
        }

        return this.run(
            msg,
            SET_GAS
        )
    }
}

export class ViewingKeyComponentExecutor extends Executor {
    async create_viewing_key(
        padding?: string | null,
    ): Promise<{result: Tx, key: ViewingKey}> {
        const msg = {
            auth: {
                create_viewing_key: {
                    entropy: create_entropy(),
                    padding
                }
            }
        }

        const result = await this.run(
            msg,
            CREATE_GAS
        )

        return {
            result,
            key: result.code == 0 ? decode_vk(result) : ''
        }
    }

    async set_viewing_key(
        key: ViewingKey,
        padding?: string | null,
    ): Promise<Tx> {
        const msg = {
            auth: {
                set_viewing_key: {
                    key,
                    padding
                }
            }
        }

        return this.run(
            msg,
            SET_GAS
        )
    }
}

interface CreateViewingKeyResult {
    create_viewing_key: {
        key: ViewingKey
    }
}

function decode_vk(result: Tx): ViewingKey {
    const decode_result = decode_data<CreateViewingKeyResult>(result)
    return decode_result.create_viewing_key.key
}
