FROM node:25.3.0 AS base
WORKDIR /app

FROM base AS build-deps
COPY package.json package-lock.json ./
RUN npm install

FROM base AS run-deps
COPY package.json package-lock.json ./
RUN npm install --omit=dev

FROM build-deps AS build
COPY . .
RUN npm run build

FROM base AS runtime
COPY --from=build /app/dist ./dist
COPY --from=run-deps /app/node_modules ./node_modules

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000

CMD [ "node", "dist/server/entry.mjs" ]
