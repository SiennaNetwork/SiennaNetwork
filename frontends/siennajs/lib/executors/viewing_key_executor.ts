import { Executor } from '../contract'
import { create_entropy, decode_data, ViewingKey } from '../core'

import { ExecuteResult } from 'secretjs'

interface CreateViewingKeyResult {
    create_viewing_key: {
        key: ViewingKey
    }
}

export class ViewingKeyExecutor extends Executor {
    async create_viewing_key(
        padding?: string | null,
    ): Promise<{result: ExecuteResult, key: ViewingKey}> {
        const msg = {
            create_viewing_key: {
                entropy: create_entropy(),
                padding,
            },
        }

        const result = await this.run(
            msg,
            "200000",
        )

        const decode_result = decode_data<CreateViewingKeyResult>(result)
        const key = decode_result.create_viewing_key.key

        return { result, key }
    }

    async set_viewing_key(
        key: ViewingKey,
        padding?: string | null,
    ): Promise<ExecuteResult> {
        const msg = {
            set_viewing_key: {
                key,
                padding,
            },
        }

        return this.run(
            msg,
            "200000",
        )
    }
}
