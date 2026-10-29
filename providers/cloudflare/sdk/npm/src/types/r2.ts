import {ResourceConfig} from "@giong/sdk";

export class R2 extends ResourceConfig {
    readonly name: string;

    constructor(config: { name: string }) {
        super();
        this.name = config.name;
    }

    get type(): string {
        return "cloudflare.r2";
    }
}
