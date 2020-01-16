export default () => {

    function isFeatureEnabled(name) {
        return localStorage.getItem(`features.${name}`) === "true";
    }

    return {
        facebookLogin() {
            return isFeatureEnabled('facebook-login')
        },
        rssChannel() {
            return isFeatureEnabled('rss-channel')
        }
    }
}