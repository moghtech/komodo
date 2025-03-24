"use strict";(self.webpackChunkdocsite=self.webpackChunkdocsite||[]).push([[976],{7879:(e,o,n)=>{n.r(o),n.d(o,{assets:()=>c,contentTitle:()=>a,default:()=>h,frontMatter:()=>t,metadata:()=>r,toc:()=>d});const r=JSON.parse('{"id":"intro","title":"What is Komodo?","description":"Komodo is a web app to provide structure for managing your servers, builds, deployments, and automated procedures.","source":"@site/docs/intro.md","sourceDirName":".","slug":"/intro","permalink":"/docs/intro","draft":false,"unlisted":false,"editUrl":"https://github.com/moghtech/komodo/tree/main/docsite/docs/intro.md","tags":[],"version":"current","frontMatter":{"slug":"/intro"},"sidebar":"docs","next":{"title":"Resources","permalink":"/docs/resources"}}');var s=n(4848),i=n(8453);const t={slug:"/intro"},a="What is Komodo?",c={},d=[{value:"Docker",id:"docker",level:2},{value:"Architecture and Components",id:"architecture-and-components",level:2},{value:"Core",id:"core",level:3},{value:"Periphery",id:"periphery",level:3},{value:"Core API",id:"core-api",level:2},{value:"Permissioning",id:"permissioning",level:2}];function l(e){const o={a:"a",admonition:"admonition",code:"code",h1:"h1",h2:"h2",h3:"h3",header:"header",li:"li",p:"p",strong:"strong",ul:"ul",...(0,i.R)(),...e.components};return(0,s.jsxs)(s.Fragment,{children:[(0,s.jsx)(o.header,{children:(0,s.jsx)(o.h1,{id:"what-is-komodo",children:"What is Komodo?"})}),"\n",(0,s.jsx)(o.p,{children:"Komodo is a web app to provide structure for managing your servers, builds, deployments, and automated procedures."}),"\n",(0,s.jsx)(o.p,{children:"With Komodo you can:"}),"\n",(0,s.jsxs)(o.ul,{children:["\n",(0,s.jsx)(o.li,{children:"Connect all of your servers, and alert on CPU usage, memory usage, and disk usage."}),"\n",(0,s.jsx)(o.li,{children:"Create, start, stop, and restart Docker containers on the connected servers, and view their status and logs."}),"\n",(0,s.jsx)(o.li,{children:"Deploy docker compose stacks. The file can be defined in UI, or in a git repo, with auto deploy on git push."}),"\n",(0,s.jsx)(o.li,{children:"Build application source into auto-versioned Docker images, auto built on webhook. Deploy single-use AWS instances for infinite capacity."}),"\n",(0,s.jsx)(o.li,{children:"Manage repositories on connected servers, which can perform automation via scripting / webhooks."}),"\n",(0,s.jsx)(o.li,{children:"Manage all your configuration / environment variables, with shared global variable and secret interpolation."}),"\n",(0,s.jsx)(o.li,{children:"Keep a record of all the actions that are performed and by whom."}),"\n"]}),"\n",(0,s.jsx)(o.p,{children:'There is no limit to the number of servers you can connect, and there will never be. There is no limit to what API you can use for automation, and there never will be. No "business edition" here.'}),"\n",(0,s.jsx)(o.h2,{id:"docker",children:"Docker"}),"\n",(0,s.jsxs)(o.p,{children:["Komodo is opinionated by design, and uses ",(0,s.jsx)(o.a,{href:"https://docs.docker.com/",children:"docker"})," as the container engine for building and deploying."]}),"\n",(0,s.jsx)(o.admonition,{type:"info",children:(0,s.jsxs)(o.p,{children:["Komodo also supports ",(0,s.jsx)(o.a,{href:"https://podman.io/",children:(0,s.jsx)(o.strong,{children:"podman"})})," instead of docker by utilizing the ",(0,s.jsx)(o.code,{children:"podman"})," -> ",(0,s.jsx)(o.code,{children:"docker"})," alias.\nFor Stack / docker compose support with podman, check out ",(0,s.jsx)(o.a,{href:"https://github.com/containers/podman-compose",children:(0,s.jsx)(o.strong,{children:"podman-compose"})}),". Thanks to ",(0,s.jsx)(o.code,{children:"u/pup_kit"})," for checking this."]})}),"\n",(0,s.jsx)(o.h2,{id:"architecture-and-components",children:"Architecture and Components"}),"\n",(0,s.jsx)(o.p,{children:"Komodo is composed of a single core and any amount of connected servers running the periphery application."}),"\n",(0,s.jsx)(o.h3,{id:"core",children:"Core"}),"\n",(0,s.jsx)(o.p,{children:"Komodo Core is a web server hosting the Core API and browser UI. All user interaction with the connected servers flow through the Core."}),"\n",(0,s.jsx)(o.h3,{id:"periphery",children:"Periphery"}),"\n",(0,s.jsx)(o.p,{children:"Komodo Periphery is a small stateless web server that runs on all connected servers. It exposes an API called by Komodo Core to perform actions on the server, get system usage, and container status / logs. It is only intended to be reached from the core, and has an address whitelist to limit the IPs allowed to call this API."}),"\n",(0,s.jsx)(o.h2,{id:"core-api",children:"Core API"}),"\n",(0,s.jsxs)(o.p,{children:["Komodo exposes powerful functionality over the Core's REST and Websocket API, enabling infrastructure engineers to manage their infrastructure programmatically. There is a ",(0,s.jsx)(o.a,{href:"https://crates.io/crates/komodo_client",children:"rust crate"})," and ",(0,s.jsx)(o.a,{href:"https://www.npmjs.com/package/komodo_client",children:"npm package"})," to simplify programmatic interaction with the API, but in general this can be accomplished using any programming language that can make REST requests."]}),"\n",(0,s.jsx)(o.h2,{id:"permissioning",children:"Permissioning"}),"\n",(0,s.jsxs)(o.p,{children:["Komodo is a system designed to be used by many users, whether they are developers, operations personnel, or administrators. The ability to affect an applications state is very powerful, so Komodo has a granular permissioning system to only provide this functionality to the intended users. The permissioning system is explained in detail in the ",(0,s.jsx)(o.a,{href:"/docs/permissioning",children:"permissioning"})," section."]}),"\n",(0,s.jsxs)(o.p,{children:["User sign-on is possible using username / password, or with Oauth (Github and Google). See ",(0,s.jsx)(o.a,{href:"/docs/setup/",children:"Core Setup"}),"."]})]})}function h(e={}){const{wrapper:o}={...(0,i.R)(),...e.components};return o?(0,s.jsx)(o,{...e,children:(0,s.jsx)(l,{...e})}):l(e)}},8453:(e,o,n)=>{n.d(o,{R:()=>t,x:()=>a});var r=n(6540);const s={},i=r.createContext(s);function t(e){const o=r.useContext(i);return r.useMemo((function(){return"function"==typeof e?e(o):{...o,...e}}),[o,e])}function a(e){let o;return o=e.disableParentContext?"function"==typeof e.components?e.components(s):e.components||s:t(e.components),r.createElement(i.Provider,{value:o},e.children)}}}]);