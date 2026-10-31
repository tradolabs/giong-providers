import {env} from "node:process";
import {Provider} from "@giong/sdk";

export interface CloudflareConfig {
    accountId?: string;
    apiToken?: string;
}

export class CloudflareProvider extends Provider {
    readonly #accountId: string;
    readonly #apiToken: string;

    constructor(name: string, credentials: { accountId: string, apiToken?: string }) {
        super(name);
        this.#accountId = credentials.accountId;
        this.#apiToken = credentials.apiToken ?? env.CLOUDFLARE_API_TOKEN!;
    }

    get package(): string {
        return "cloudflare";
    }

    get inputs(): Record<string, unknown> {
        return {
            'CLOUDFLARE_ACCOUNT_ID': this.#accountId,
            "CLOUDFLARE_API_TOKEN": this.#apiToken,
        }
    }
}
