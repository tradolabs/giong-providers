import {Resource} from "@giong/sdk";
import {CloudflareProvider} from "../provider.ts";

export type R2Config = {}


export class R2 extends Resource<CloudflareProvider> {
    private config: R2Config;

    constructor(name: string, config: R2Config, provider: CloudflareProvider) {
        super(name, provider);
        this.config = config;
    }

    get package(): string {
        return "cloudflare/r2";
    }

    get inputs(): Record<string, unknown> {
        return {name: this.name};
    }
}
