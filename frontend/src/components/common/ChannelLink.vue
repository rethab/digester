<template>
  <div>
    <a
      v-if="channelLink"
      :href="channelLink"
      target="_blank"
      style="text-decoration:none; word-break: break-word; color: inherit"
    >{{text}}</a>
    <router-link
      v-else
      :to="`/list/${channelId}`"
      style="text-decoration:none; word-break: break-word; color: inherit"
    >{{text}}</router-link>
  </div>
</template>

<script>
export default {
  props: {
    channelId: {
      type: Number
    },
    channelLink: {
      type: String
    },
    text: {
      type: String,
      required: true
    }
  }
};
</script>