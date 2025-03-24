"use strict";(self.webpackChunkdocsite=self.webpackChunkdocsite||[]).push([[636],{7700:(e,o,n)=>{n.r(o),n.d(o,{assets:()=>a,contentTitle:()=>c,default:()=>h,frontMatter:()=>r,metadata:()=>s,toc:()=>l});const s=JSON.parse('{"id":"docker-compose","title":"Docker Compose","description":"Komodo can deploy docker compose projects through the Stack resource.","source":"@site/docs/docker-compose.md","sourceDirName":".","slug":"/docker-compose","permalink":"/docs/docker-compose","draft":false,"unlisted":false,"editUrl":"https://github.com/moghtech/komodo/tree/main/docsite/docs/docker-compose.md","tags":[],"version":"current","frontMatter":{},"sidebar":"docs","previous":{"title":"Container Management","permalink":"/docs/deploy-containers/lifetime-management"},"next":{"title":"Variables and Secrets","permalink":"/docs/variables"}}');var t=n(4848),i=n(8453);const r={},c="Docker Compose",a={},l=[{value:"Define the compose file/s",id:"define-the-compose-files",level:2},{value:"Importing Existing Compose projects",id:"importing-existing-compose-projects",level:2},{value:"Pass Environment Variables",id:"pass-environment-variables",level:2}];function d(e){const o={admonition:"admonition",code:"code",h1:"h1",h2:"h2",header:"header",li:"li",ol:"ol",p:"p",strong:"strong",ul:"ul",...(0,i.R)(),...e.components};return(0,t.jsxs)(t.Fragment,{children:[(0,t.jsx)(o.header,{children:(0,t.jsx)(o.h1,{id:"docker-compose",children:"Docker Compose"})}),"\n",(0,t.jsxs)(o.p,{children:["Komodo can deploy docker compose projects through the ",(0,t.jsx)(o.code,{children:"Stack"})," resource."]}),"\n",(0,t.jsx)(o.h2,{id:"define-the-compose-files",children:"Define the compose file/s"}),"\n",(0,t.jsx)(o.p,{children:"Komodo supports 3 ways of defining the compose files:"}),"\n",(0,t.jsxs)(o.ol,{children:["\n",(0,t.jsxs)(o.li,{children:[(0,t.jsx)(o.strong,{children:"Write them in the UI"}),", and Komodo will write them to your host at deploy-time."]}),"\n",(0,t.jsxs)(o.li,{children:[(0,t.jsx)(o.strong,{children:"Store the files anywhere on the host"}),", and Komodo will just run the compose commands on the existing files."]}),"\n",(0,t.jsxs)(o.li,{children:[(0,t.jsx)(o.strong,{children:"Store them in a git repo"}),", and have Komodo clone it on the host to deploy."]}),"\n"]}),"\n",(0,t.jsx)(o.p,{children:"If you manage your compose files in git repos:"}),"\n",(0,t.jsxs)(o.ul,{children:["\n",(0,t.jsx)(o.li,{children:"All your files, across all servers, are available locally to edit in your favorite text editor."}),"\n",(0,t.jsx)(o.li,{children:"All of your changes are tracked, and can be reverted."}),"\n",(0,t.jsxs)(o.li,{children:["You can use the git webhooks to do other automations when you change the compose file contents. Redeploying will be as easy as ",(0,t.jsx)(o.code,{children:"git push"}),"."]}),"\n"]}),"\n",(0,t.jsx)(o.admonition,{type:"info",children:(0,t.jsx)(o.p,{children:"Many Komodo resources need access to git repos. There is an in-built token management system (managed in UI or in config file) to give resources access to credentials.\nAll resources which depend on git repos are able to use these credentials to access private repos."})}),"\n",(0,t.jsx)(o.h2,{id:"importing-existing-compose-projects",children:"Importing Existing Compose projects"}),"\n",(0,t.jsx)(o.p,{children:"First create the Stack in Komodo, and ensure it has access to the compose files using one\nof the three methods above. Make sure to attach the server you wish to deploy on."}),"\n",(0,t.jsxs)(o.p,{children:['In order for Komodo to pick up a running project, it has to know the compose "project name".\nYou can find the project name by running ',(0,t.jsx)(o.code,{children:"docker compose ls"})," on the host."]}),"\n",(0,t.jsx)(o.p,{children:'By default, Komodo will assume the Stack name is the compose project name.\nIf this is different than the project name on the host, you can configure a custom "Project Name" in the config.'}),"\n",(0,t.jsx)(o.h2,{id:"pass-environment-variables",children:"Pass Environment Variables"}),"\n",(0,t.jsx)(o.p,{children:"Komodo is able to pass custom environment variables to the docker compose process.\nThis works by:"}),"\n",(0,t.jsxs)(o.ol,{children:["\n",(0,t.jsx)(o.li,{children:'Write the variables to a ".env" file on the host at deploy-time.'}),"\n",(0,t.jsxs)(o.li,{children:["Pass the file to docker compose using the ",(0,t.jsx)(o.code,{children:"--env-file"})," flag."]}),"\n"]}),"\n",(0,t.jsx)(o.admonition,{type:"info",children:(0,t.jsxs)(o.p,{children:["Just like all other resources with Environments (Deployments, Repos, Builds),\nStack Environments support ",(0,t.jsx)(o.strong,{children:"Variable and Secret interpolation"}),". Define global variables\nin the UI and share the values across environments."]})})]})}function h(e={}){const{wrapper:o}={...(0,i.R)(),...e.components};return o?(0,t.jsx)(o,{...e,children:(0,t.jsx)(d,{...e})}):d(e)}},8453:(e,o,n)=>{n.d(o,{R:()=>r,x:()=>c});var s=n(6540);const t={},i=s.createContext(t);function r(e){const o=s.useContext(i);return s.useMemo((function(){return"function"==typeof e?e(o):{...o,...e}}),[o,e])}function c(e){let o;return o=e.disableParentContext?"function"==typeof e.components?e.components(t):e.components||t:r(e.components),s.createElement(i.Provider,{value:o},e.children)}}}]);