import { resolve, dirname } from 'path'
import { writeFileSync, existsSync, mkdirSync } from 'fs'

export interface IJsonFileWriter {
    base_path: string
    write(value: any, file_name: string): void
}

export class NullJsonFileWriter implements IJsonFileWriter {
    base_path: string;

    constructor() {
        this.base_path = ''
    }

    write(_value: any, _file_name: string) { }
}

export class JsonFileWriter implements IJsonFileWriter {
    constructor(public base_path: string) { }

    write(value: any, file_name: string) {
        const path = resolve(this.base_path, `${file_name}.json`)
        const dir = dirname(path)

        if(!existsSync(dir)) {
            mkdirSync(dir, { recursive: true })
        }

        writeFileSync(path, JSON.stringify(value, null, 2))
    }
}
