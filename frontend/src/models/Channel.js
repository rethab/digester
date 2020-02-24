'use strict'

export default class Channel {
    constructor(type, name) {
        this.type = type;
        this.name = name;
    }

    isRss() {
        return this.type === "RssFeed";
    }

    isGithubRelease() {
        return this.type === "GithubRelease";
    }

    label() {
        if (this.isGithubRelease()) {
            return "Repository";
        } else if (this.isRss()) {
            return "Url";
        } else {
            throw `Unknown type ${this.type}`;
        }
    }

    validate() {
        let errors = [];
        if (this.isGithubRelease()) {
            // if a user enters a github url, we help them a bit by
            // extracting the repository
            let repoWithUrl = /^.*github\.com.*\/([^/]+\/[^/]+)$/.exec(this.name);
            if (repoWithUrl) {
                this.name = repoWithUrl[1];
            }
            if (!/^[^/]+\/[^/]+$/.test(this.name)) {
                errors.push("Format: author/repository");
            }
        } else if (this.isRss()) {
            if (!/^.*[^.]+\.[^.]+.*$/.test(this.name)) {
                errors.push("Format: theverge.com");
            }
        } else {
            throw `Unknwon channel type: ${this.type}`;
        }

        return errors;
    }

    show() {
        return `${this.type} / ${this.name}`;
    }

}