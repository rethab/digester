export default () => {

    function get(name) {
        return localStorage.getItem(`preferences.${name}`);
    }

    function set(name, value) {
        return localStorage.setItem(`preferences.${name}`, value);
    }

    return {
        get channelType() {
            return get('channelType')
        },
        set channelType(value) {
            return set('channelType', value)
        },
        frequency: {
            get frequency() {
                return get('frequency.frequency')
            },
            set frequency(value) {
                return set('frequency.frequency', value)
            },
            get day() {
                return get('frequency.day')
            },
            set day(value) {
                return set('frequency.day', value)
            },
            get time() {
                return get('frequency.time')
            },
            set time(value) {
                return set('frequency.time', value)
            },
        }
    }
}