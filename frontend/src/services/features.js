export default () => {

    function isFeatureEnabled(name) {
        return localStorage.getItem(`features.${name}`) === "true";
    }

    return {
        rssChannel() {
            return isFeatureEnabled('rss-channel')
        }
    }
}