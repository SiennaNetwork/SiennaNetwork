import { ExecuteResult } from 'secretjs'

export type ValueType = number | string | bigint | undefined | boolean

export interface AsyncFn {
    (): Promise<void>
}

export function assert(condition: boolean) {
    if(!condition) {
        throw new Error('Assertion failed!')
    }
}

export function assert_equal(val1: ValueType, val2: ValueType) {
    if (val1 !== val2) {
        throw new Error(`assert_equal failed: ${val1} !== ${val2}`)
    }
}

export function assert_not_equal(val1: ValueType, val2: ValueType) {
    if (val1 === val2) {
        throw new Error(`assert_not_equal failed: ${val1} === ${val2}`)
    }
}

export function assert_objects_equal(object1: any, object2: any) {
    if(!assert_objects_equal_internal(object1, object2)) {
        console.log('assert_objects_equal failed:\n Object 1:')
        print_object(object1)
        console.log('\nObject 2:')
        print_object(object2)

        throw new Error()
    }
}
  
export async function execute_test(test_name: string, test: AsyncFn) {
    try {
        await test()
        print_success(test_name)
    } catch(e) {
        console.error(e)
        print_error(test_name)
    }
}
  
export async function execute_test_expect(
      test_name: string,
      test: AsyncFn,
      expected_error: string
) {
    try {
        await test()
        print_error(`${test_name}(expected error)`)
    } catch (e) {
        if (e.message.includes(expected_error)) {
            print_success(test_name)
            return
        }
  
        console.error(e)
        print_error(test_name)
    }
}

// This shouldn't be changed as it is taken from the frontend and it is used to test
// whether it works with the contracts' logs
export function extract_log_value(txResult: ExecuteResult, key: string): string | undefined {
    return txResult?.logs[0]?.events?.find(e => e.type === 'wasm')?.attributes?.find(a => a.key === key)?.value
}

export function print_object(object: any) {
    console.log(JSON.stringify(object, null, 2))
}

function assert_objects_equal_internal(object1: any, object2: any) {
    const keys1 = Object.keys(object1);
    const keys2 = Object.keys(object2);
  
    if (keys1.length !== keys2.length) {
        return false;
    }
  
    for (const key of keys1) {
        const val1 = object1[key];
        const val2 = object2[key];
        const areObjects = isObject(val1) && isObject(val2);
        if (
            areObjects && !assert_objects_equal_internal(val1, val2) ||
            !areObjects && val1 !== val2
        ) {
            return false
        }
    }

    return true
}

function print_success(test_name: string) {
    console.log(`${test_name}..............................✅`)
}
  
function print_error(test_name: string) {
    console.log(`${test_name}..............................❌`)
}

function isObject(object: any) {
    return object != null && typeof object === 'object';
}
