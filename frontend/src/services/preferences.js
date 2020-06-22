export default () => {

    function get(name) {
        return localStorage.getItem(`preferences.${name}`);
    }

    function set(name, value) {
        return localStorage.setItem(`preferences.${name}`, value);
    }

    return {
        getChannelType() {
            return get('channelType')
        },
        setChannelType(value) {
            return set('channelType', value)
        },
    }
}