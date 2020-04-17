'use strict'

export default class Channel {
    constructor(type, name) {
        this.type = type;
        this.name = name;
    }

    static get Rss() {
        return "RssFeed";
    }

    static get GithubRelease() {
        return "GithubRelease";
    }

    static get Twitter() {
        return "Twitter";
    }

    static get List() {
        return "List";
    }

    isRss() {
        return this.type === Channel.Rss;
    }

    isGithubRelease() {
        return this.type === Channel.GithubRelease;
    }

    isTwitter() {
        return this.type === Channel.Twitter;
    }

    isList() {
        return this.type === Channel.List;
    }

    label() {
        if (this.isGithubRelease()) {
            return "Repository";
        } else if (this.isRss()) {
            return "Url";
        } else if (this.isTwitter()) {
            return "Twitter User";
        } else if (this.isList()) {
            return "Name of the List";
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
        } else if (this.isList()) {
            if (!this.name || this.name < 3) {
                errors.push("Enter the name of a List")
            }
        } else if (this.isTwitter()) {
            if (!this.name || this.name > 20) {
                errors.push("Twitter names are not that long")
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