import { resolve, dirname } from 'path';
import { writeFileSync, existsSync, mkdirSync } from 'fs';
export class NullJsonFileWriter {
    constructor() {
        this.base_path = '';
    }
    write(_value, _file_name) { }
}
export class JsonFileWriter {
    constructor(base_path) {
        this.base_path = base_path;
    }
    write(value, file_name) {
        const path = resolve(this.base_path, `${file_name}.json`);
        const dir = dirname(path);
        if (!existsSync(dir)) {
            mkdirSync(dir, { recursive: true });
        }
        writeFileSync(path, JSON.stringify(value, null, 2));
    }
}
//# sourceMappingURL=json_file_writer.js.map