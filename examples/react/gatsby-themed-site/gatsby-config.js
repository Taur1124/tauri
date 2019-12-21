module.exports = {
  siteMetadata: {
    siteTitle: `Jacob Bolda`,
    siteDescription: `Structural Engineer with a knack for creative solutions using code and ingenuity.`,
    siteAuthor: `Jacob Bolda`,
    siteAuthorIdentity: `Structural Engineer`,
    siteLanding: `
    Focusing on the intersection of tech and Structural
    Engineering. Masters degree in Structural Engineering
    from the Milwaukee School of Engineering, undergrad in
    Architectural Engineering with a minor in Management,
    and a deep understanding of software and programming.
    Marrying that experience with problem solving and
    systematizing is powerful.
  `,
    siteContact: "https://twitter.com/jacobbolda",
    contactLinks: [
      {
        url: "mailto:me@jacobbolda.com",
        text: "me@jacobbolda.com",
        icon: ["far", "envelope"]
      },
      {
        url: "https://twitter.com/jacobbolda",
        text: "@jacobbolda",
        icon: ["fab", "twitter"]
      },
      {
        url: "https://linkedin.com/in/bolda",
        text: "linkedin.com/in/bolda",
        icon: ["fab", "linkedin"]
      },
      {
        url: "https://github.com/jbolda",
        text: "github.com/jbolda",
        icon: ["fab", "github"]
      },
      {
        url: "https://keybase.io/jbolda",
        text: "keybase.io/jbolda",
        icon: ["fab", "keybase"]
      },
      {
        url: "https://angel.co/jacobbolda",
        text: "angel.co/jacobbolda",
        icon: ["fab", "angellist"]
      },
      {
        url: "http://www.jbolda.com/photo",
        text: "My Photographs",
        icon: ["fas", "camera"]
      }
    ],
    navLinks: [{ url: "/recipes/", text: "Our Recipes" }]
  },
  plugins: [
    {
      resolve: `gatsby-source-filesystem`,
      options: {
        name: `articles`,
        path: `${__dirname}/src/articles/`
      }
    },
    `gatsby-plugin-theme-ui`,
    `gatsby-plugin-sharp`,
    `gatsby-transformer-sharp`,
    `@jbolda/gatsby-theme-homepage`,
    `@jbolda/gatsby-theme-articles`,
    {
      resolve: `gatsby-plugin-mdx`,
      options: {}
    },
    `gatsby-plugin-react-helmet`,
    `gatsby-plugin-netlify`
  ]
};
