import {ResourceConfig} from "@tradolabs/giong";

export class R2 extends ResourceConfig {
    readonly name: string;

    constructor(config: {name: string}) {
        super("cloudflare.r2");
        this.name = config.name;
    }
}
