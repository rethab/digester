<template>
  <div>
    <v-row>
      <v-col class="text-center">
        <p class="title">You are in the Cockpit</p>
      </v-col>
    </v-row>

    <v-row>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Subscribe"
          :lighten="4"
          link="/subs"
          subtitle="to Twitter Feeds, Blogs, etc"
          text="Receive weekly (or daily) E-mails with all new entries from blogs you have subscribed to."
        />
      </v-col>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Preview Updates"
          :lighten="4"
          link="/updates"
          subtitle="from your subscriptions"
          text="This is for the impatient! If you can't wait to receive your digest via E-mail, this is where to look."
        />
      </v-col>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="My Lists"
          :lighten="5"
          link="/lists"
          subtitle="bundle channels by topic"
          text="Group Blogs, Twitter feeds, etc in Lists and share them with friends."
        />
      </v-col>
      <v-col cols="12" sm="6">
        <CockpitCard
          title="Settings"
          :lighten="5"
          link="/settings"
          subtitle="for your Digester account"
          text="Are you travelling and need to update your timezone?"
        />
      </v-col>
    </v-row>
  </div>
</template>

<script>
import CockpitCard from "@/components/CockpitCard.vue";
export default {
  components: {
    CockpitCard
  }
};
</script>