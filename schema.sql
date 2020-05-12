drop database whatsoo;
create database whatsoo;
create table whatsoo.user
(
	pk_id int auto_increment comment '主键ID'
		primary key,
	uk_username varchar(50) not null comment '用户名',
	uk_e_mail varchar(50) not null comment 'E_Mail',
	avatar varchar(100) null default 'https://avatars3.githubusercontent.com/u/18442141' comment '头像',
	blog_url varchar(100) null comment '博客网址',
	introduce varchar(500) null comment '自我介绍',
	github_uid varchar(50) null comment 'Github用户名',
	create_time timestamp not null comment '创建时间',
	update_time timestamp not null comment '更新时间',
	constraint user_uk_e_mail_uindex
		unique (uk_e_mail),
	constraint user_uk_username_uindex
		unique (uk_username)
)
comment '用户表';