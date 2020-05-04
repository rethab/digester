<template>
  <div>
    <v-radio-group v-model="channel.type" :mandatory="true" row class="mt-0">
      <v-row dense justify="space-between">
        <v-col md="3" cols="6" v-for="(type, idx) in types" :key="idx" class="px-0 py-2">
          <v-radio
            v-on:input="$emit('selectChannel', channel)"
            :label="type.text"
            :value="type.value"
          ></v-radio>
        </v-col>
      </v-row>
    </v-radio-group>
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
      { text: "Twitter", value: Channel.Twitter },
      { text: "Blog / Rss", value: Channel.Rss },
      { text: "Github", value: Channel.GithubRelease }
    ];

    if (this.showList) {
      types.push({ text: "List", value: Channel.List });
    }

    return {
      channel: new Channel(this.value.type, this.value.name),
      types: types
    };
  }
};
</script>