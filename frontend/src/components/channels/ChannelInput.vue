<template>
  <div>
    <v-row dense>
      <v-col cols="12">
        <v-select
          :items="types"
          v-model="channel.type"
          v-on:input="$emit('selectChannel', channel)"
          label="Type"
          append-icon
          :disabled="types.length === 1"
        ></v-select>
      </v-col>
    </v-row>
    <v-row dense>
      <v-col cols="12">
        <v-text-field
          v-on:input="$emit('selectChannel', channel)"
          v-model.trim="channel.name"
          :error-messages="nameErrors"
          :label="channel.label()"
        ></v-text-field>
      </v-col>
    </v-row>
  </div>
</template>

<script>
import Channel from "@/models/Channel.js";
export default {
  props: {
    value: {
      type: Object,
      required: true
    },
    nameErrors: {
      type: Array,
      required: true
    },
    showList: {
      type: Boolean,
      default: false
    }
  },
  data() {
    let types = [
      { text: "Blog / News", value: Channel.Rss },
      { text: "Github", value: Channel.GithubRelease }
    ];

    if (this.showList) {
      types.unshift({ text: "List", value: Channel.List });
    }

    return {
      channel: new Channel(this.value.type, this.value.name),
      types: types
    };
  }
};
</script>