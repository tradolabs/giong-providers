import {Resource} from "@giong/sdk";
import {CloudflareProvider} from "../provider.ts";

export type R2Config = Record<string, never>;

export class R2 extends Resource<CloudflareProvider> {
    constructor(name: string, _config: R2Config, provider: CloudflareProvider) {
        super(name, provider);
    }

    get package(): string {
        return "cloudflare/r2";
    }

    get inputs(): Record<string, unknown> {
        return {name: this.name};
    }
}
