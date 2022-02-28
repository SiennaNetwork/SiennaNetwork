import { Executor } from '../contract'
import { create_entropy, decode_data, ViewingKey } from '../core'

import { ExecuteResult } from 'secretjs'

const CREATE_GAS = '50000'
const SET_GAS = '40000'

export class ViewingKeyExecutor extends Executor {
    async create_viewing_key(
        padding?: string | null,
    ): Promise<{result: ExecuteResult, key: ViewingKey}> {
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
            key: decode_vk(result)
        }
    }

    async set_viewing_key(
        key: ViewingKey,
        padding?: string | null,
    ): Promise<ExecuteResult> {
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
    ): Promise<{result: ExecuteResult, key: ViewingKey}> {
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
            key: decode_vk(result)
        }
    }

    async set_viewing_key(
        key: ViewingKey,
        padding?: string | null,
    ): Promise<ExecuteResult> {
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

function decode_vk(result: ExecuteResult): ViewingKey {
    const decode_result = decode_data<CreateViewingKeyResult>(result)
    return decode_result.create_viewing_key.key
}
