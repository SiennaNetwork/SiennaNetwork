export * from './auth'
export * from './overseer'
export * from './market'

export interface PaginatedResponse<T> {
    entries: T[],
    /**
     * The total number of entries stored by the contract.
     */
    total: number
}
